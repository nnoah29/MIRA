//! Pont asynchrone : tâches des requêtes, flux d'événements et file de messages de la boucle.

use std::sync::Arc;
use std::time::Duration;

use crossterm::event::KeyEvent;
use mira_assembly::api::Engine;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::task::{JoinHandle, JoinSet};
use tokio::time::Interval;

use super::event_feed::spawn_event_feed;
use super::request_task::spawn_request;
use crate::commands::Command;
use crate::update::Msg;

const DRAIN_DEADLINE: Duration = Duration::from_secs(3);

/// Ce qui réveille la boucle.
#[allow(clippy::large_enum_variant)]
pub(crate) enum Wake {
    Key(KeyEvent),
    Msg(Msg),
    Tick,
    KeysClosed,
}

/// Possède les tâches en vol et le flux d'événements ; leurs clones du moteur meurent avec elles.
pub(crate) struct Runtime {
    engine: Arc<Engine>,
    sender: UnboundedSender<Msg>,
    receiver: UnboundedReceiver<Msg>,
    tasks: JoinSet<()>,
    feed: JoinHandle<()>,
}

impl Runtime {
    /// Ouvre la file de messages et y branche les événements du moteur.
    pub(crate) fn start(engine: Arc<Engine>) -> Runtime {
        let (sender, receiver) = mpsc::unbounded_channel();
        let feed = spawn_event_feed(&engine, sender.clone());
        Runtime {
            engine,
            sender,
            receiver,
            tasks: JoinSet::new(),
            feed,
        }
    }

    /// Lance les requêtes demandées et oublie les tâches finies ; vrai si une commande demande la sortie.
    pub(crate) fn run(&mut self, commands: Vec<Command>) -> bool {
        let mut quit = false;
        for command in commands {
            match command {
                Command::Execute { tag, request } => {
                    spawn_request(&mut self.tasks, &self.engine, &self.sender, tag, request);
                }
                Command::Quit => quit = true,
            }
        }
        while self.tasks.try_join_next().is_some() {}
        quit
    }

    /// Attend une touche, un message du pont ou le battement de redessin.
    pub(crate) async fn next_wake(
        &mut self,
        keys: &mut UnboundedReceiver<KeyEvent>,
        tick: &mut Interval,
    ) -> Wake {
        tokio::select! {
            key = keys.recv() => key.map_or(Wake::KeysClosed, Wake::Key),
            Some(msg) = self.receiver.recv() => Wake::Msg(msg),
            _ = tick.tick() => Wake::Tick,
        }
    }

    /// Arrête le flux, laisse finir les requêtes en vol (délai borné), interrompt les dernières et rend le moteur.
    pub(crate) async fn stop(mut self) {
        self.feed.abort();
        let _ = (&mut self.feed).await;
        let drain = async { while self.tasks.join_next().await.is_some() {} };
        let _ = tokio::time::timeout(DRAIN_DEADLINE, drain).await;
        self.tasks.shutdown().await;
    }
}
