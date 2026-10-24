//! Bus réel : persistance numérotée dans la base du profil, puis diffusion `broadcast`.

use std::sync::Arc;

use mira_vocabulary::{Event, EventFilter, EventKind, MiraError};
use tokio::sync::broadcast;

use super::counters::{Announced, may_change_counters};
use super::sink::EventSink;
use super::subscription::EventSubscription;
use crate::ports::Clock;
use crate::storage::Database;

const LIVE_CAPACITY: usize = 1024;

/// Bus d'événements d'un moteur ouvert sur un profil.
#[derive(Debug, Clone)]
pub struct EventBus {
    pub(super) db: Arc<Database>,
    clock: Arc<dyn Clock>,
    live: broadcast::Sender<Event>,
    pub(super) announced: Announced,
}

impl EventBus {
    /// Crée le bus d'un profil ; les compteurs actuels des boîtes sont la référence des annonces
    pub fn new(db: Arc<Database>, clock: Arc<dyn Clock>) -> Self {
        let (live, _) = broadcast::channel(LIVE_CAPACITY);
        let announced = Announced::default();
        announced.seed(&db);
        Self {
            db,
            clock,
            live,
            announced,
        }
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

    /// Écrit l'événement en base (numéro croissant), le diffuse, et retient les compteurs qu'il annonce
    pub(super) fn emit_plain(&self, kind: EventKind) -> Result<Event, MiraError> {
        let at = self.clock.now();
        let event = self.db.write(|tx| tx.append_event(at, &kind))?;
        if let EventKind::MailboxCountersUpdated {
            mailbox,
            total,
            unread,
            ..
        } = &kind
        {
            self.announced.remember(mailbox, *total, *unread);
        }
        let _ = self.live.send(event.clone());
        Ok(event)
    }
}

impl EventSink for EventBus {
    /// Annonce d'abord les compteurs qui ont changé, puis écrit et diffuse l'événement
    fn emit(&self, kind: EventKind) -> Result<Event, MiraError> {
        if let (true, Some(account)) = (may_change_counters(&kind), kind.account()) {
            self.announce_changed_counters(account)?;
        }
        self.emit_plain(kind)
    }
}
