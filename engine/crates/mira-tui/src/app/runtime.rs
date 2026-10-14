//! Pont asynchrone : tâches des requêtes, flux d'événements, signaux d'arrêt et file de messages de la boucle.

use std::sync::Arc;

use crossterm::event::KeyEvent;
use mira_assembly::api::Engine;
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tokio::task::JoinHandle;
use tokio::time::Interval;

use super::event_feed::spawn_event_feed;
use super::request_task::RequestTasks;
use crate::commands::Command;
use crate::update::Msg;

/// Ce qui réveille la boucle.
#[allow(clippy::large_enum_variant)]
pub(crate) enum Wake {
    Key(KeyEvent),
    Msg(Msg),
    Settled,
    Tick,
    Stop,
    KeysClosed,
}

/// Possède les requêtes en vol et le flux d'événements ; leurs clones du moteur meurent avec eux.
pub(crate) struct Runtime {
    engine: Arc<Engine>,
    receiver: UnboundedReceiver<Msg>,
    requests: RequestTasks,
    feed: JoinHandle<()>,
}

impl Runtime {
    /// Ouvre la file de messages et y branche les événements du moteur.
    pub(crate) fn start(engine: Arc<Engine>) -> Runtime {
        let (sender, receiver) = mpsc::unbounded_channel();
        let feed = spawn_event_feed(&engine, sender.clone());
        Runtime {
            engine,
            receiver,
            requests: RequestTasks::new(sender),
            feed,
        }
    }

    /// Lance les requêtes demandées ; vrai si une commande demande la sortie.
    pub(crate) fn run(&mut self, commands: Vec<Command>) -> bool {
        let mut quit = false;
        for command in commands {
            match command {
                Command::Execute { tag, request } => {
                    let engine = Arc::clone(&self.engine);
                    self.requests
                        .spawn(tag, async move { engine.execute(request).await });
                }
                Command::Quit => quit = true,
            }
        }
        quit
    }

    /// Attend une touche, un message, la fin d'une requête, une demande d'arrêt ou le battement.
    pub(crate) async fn next_wake(
        &mut self,
        keys: &mut UnboundedReceiver<KeyEvent>,
        stop: &mut Option<UnboundedReceiver<()>>,
        tick: &mut Interval,
    ) -> Wake {
        tokio::select! {
            key = keys.recv() => key.map_or(Wake::KeysClosed, Wake::Key),
            Some(msg) = self.receiver.recv() => Wake::Msg(msg),
            Some(()) = self.requests.settle_next() => Wake::Settled,
            Some(()) = stop_requested(stop) => Wake::Stop,
            _ = tick.tick() => Wake::Tick,
        }
    }

    /// Arrête le flux puis les requêtes, et rend ainsi le moteur.
    pub(crate) async fn stop(self) {
        self.feed.abort();
        let _ = self.feed.await;
        self.requests.stop().await;
    }
}

/// Prochaine demande d'arrêt ; sans source, n'arrive jamais.
async fn stop_requested(stop: &mut Option<UnboundedReceiver<()>>) -> Option<()> {
    match stop {
        Some(receiver) => receiver.recv().await,
        None => std::future::pending().await,
    }
}
