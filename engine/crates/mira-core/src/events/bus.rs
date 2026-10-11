//! Bus réel : persistance numérotée dans la base du profil, puis diffusion `broadcast`.

use std::sync::Arc;

use mira_vocabulary::{Event, EventFilter, EventKind, MiraError};
use tokio::sync::broadcast;

use super::sink::EventSink;
use super::subscription::EventSubscription;
use crate::ports::Clock;
use crate::storage::Database;

const LIVE_CAPACITY: usize = 1024;

/// Bus d'événements d'un moteur ouvert sur un profil.
#[derive(Debug, Clone)]
pub struct EventBus {
    db: Arc<Database>,
    clock: Arc<dyn Clock>,
    live: broadcast::Sender<Event>,
}

impl EventBus {
    /// Crée le bus d'un profil
    pub fn new(db: Arc<Database>, clock: Arc<dyn Clock>) -> Self {
        let (live, _) = broadcast::channel(LIVE_CAPACITY);
        Self { db, clock, live }
    }

    /// Ouvre un abonnement qui rattrape depuis `filter.after` puis suit le direct
    pub fn subscribe(&self, filter: EventFilter) -> EventSubscription {
        EventSubscription::new(Arc::clone(&self.db), self.live.subscribe(), filter)
    }

    /// Relit en base au plus `limit` événements correspondant au filtre
    pub fn catch_up(&self, filter: &EventFilter, limit: u32) -> Result<Vec<Event>, MiraError> {
        let events = self.db.read(|tx| tx.events_after(filter.after, limit))?;
        Ok(events
            .into_iter()
            .filter(|event| filter.matches(event))
            .collect())
    }
}

impl EventSink for EventBus {
    /// Écrit l'événement en base (numéro croissant) puis le diffuse aux abonnés
    fn emit(&self, kind: EventKind) -> Result<Event, MiraError> {
        let at = self.clock.now();
        let event = self.db.write(|tx| tx.append_event(at, &kind))?;
        let _ = self.live.send(event.clone());
        Ok(event)
    }
}
