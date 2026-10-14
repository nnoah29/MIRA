//! Relais des événements du moteur vers la boucle d'événements de l'application.

use mira_assembly::api::Engine;
use mira_assembly::mira_vocabulary::EventFilter;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;

use crate::update::Msg;

/// Lance la tâche qui transmet chaque événement du moteur en `Msg::Event` ; elle s'arrête avec le moteur ou le récepteur.
pub fn spawn_event_feed(engine: &Engine, sender: UnboundedSender<Msg>) -> JoinHandle<()> {
    let mut subscription = engine.subscribe(EventFilter::default());
    tokio::spawn(async move {
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
    })
}
