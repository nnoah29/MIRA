//! Relais des événements du moteur vers la boucle d'événements de l'application.

use std::future::Future;

use mira_assembly::api::Engine;
use mira_assembly::mira_vocabulary::{EventFilter, EventSeq};
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;

use crate::update::Msg;

/// Lance la tâche qui transmet chaque événement du moteur en `Msg::Event` ; elle s'arrête avec le moteur ou le récepteur.
pub fn spawn_event_feed(engine: &Engine, sender: UnboundedSender<Msg>) -> JoinHandle<()> {
    tokio::spawn(forward_events(engine, None, sender))
}

/// Comme `spawn_event_feed`, mais seulement pour les événements postérieurs à ce numéro (pas de rejeu, AG-41).
pub fn spawn_event_feed_after(
    engine: &Engine,
    after: Option<EventSeq>,
    sender: UnboundedSender<Msg>,
) -> JoinHandle<()> {
    tokio::spawn(forward_events(engine, after, sender))
}

/// S'abonne après ce numéro et rend la boucle qui transmet les événements tant que moteur et récepteur vivent.
fn forward_events(
    engine: &Engine,
    after: Option<EventSeq>,
    sender: UnboundedSender<Msg>,
) -> impl Future<Output = ()> + Send + 'static {
    let mut subscription = engine.subscribe(EventFilter {
        after,
        ..EventFilter::default()
    });
    async move {
        loop {
            tokio::select! {
                next = subscription.next() => {
                    let Some(event) = next else { break };
                    if sender.send(Msg::Event(event)).is_err() {
                        break;
                    }
                }
                () = sender.closed() => break,
            }
        }
    }
}
