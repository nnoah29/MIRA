//! Abonnement : rattrapage depuis la base, puis direct, sans doublon ni trou.

use std::collections::VecDeque;
use std::sync::Arc;

use mira_vocabulary::{Event, EventFilter, EventSeq};
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;

use crate::storage::Database;

const BACKLOG_PAGE: u32 = 500;

/// Flux d'événements d'un client ; `next` rend les événements dans l'ordre des numéros.
#[derive(Debug)]
pub struct EventSubscription {
    db: Arc<Database>,
    live: broadcast::Receiver<Event>,
    filter: EventFilter,
    backlog: VecDeque<Event>,
    last_seen: Option<EventSeq>,
    caught_up: bool,
}

impl EventSubscription {
    /// Prépare un abonnement ; le direct est écouté avant la lecture du rattrapage
    pub(crate) fn new(
        db: Arc<Database>,
        live: broadcast::Receiver<Event>,
        filter: EventFilter,
    ) -> Self {
        let last_seen = filter.after;
        Self {
            db,
            live,
            filter,
            backlog: VecDeque::new(),
            last_seen,
            caught_up: false,
        }
    }

    /// Attend le prochain événement correspondant au filtre (`None` si le bus est fermé)
    pub async fn next(&mut self) -> Option<Event> {
        loop {
            let event = match self.next_from_backlog() {
                Some(event) => event,
                None => self.next_from_live().await?,
            };
            if self.accept(&event) {
                return Some(event);
            }
        }
    }

    /// Prend le prochain événement du rattrapage, en relisant la base si besoin
    fn next_from_backlog(&mut self) -> Option<Event> {
        if self.backlog.is_empty() && !self.caught_up {
            let page = self
                .db
                .read(|tx| tx.events_after(self.last_seen, BACKLOG_PAGE))
                .unwrap_or_default();
            self.caught_up = page.is_empty();
            self.backlog.extend(page);
        }
        self.backlog.pop_front()
    }

    /// Reçoit le prochain événement en direct ; un retard repasse par le rattrapage
    async fn next_from_live(&mut self) -> Option<Event> {
        match self.live.recv().await {
            Ok(event) => Some(event),
            Err(RecvError::Lagged(_)) => {
                self.caught_up = false;
                self.next_from_backlog()
            }
            Err(RecvError::Closed) => None,
        }
    }

    /// Écarte les événements déjà vus, puis applique le filtre
    fn accept(&mut self, event: &Event) -> bool {
        if self.last_seen.is_some_and(|seen| event.seq <= seen) {
            return false;
        }
        self.last_seen = Some(event.seq);
        self.filter.matches(event)
    }
}
