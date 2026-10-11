//! Enregistreur d'événements en mémoire, pour les tests des modules qui en émettent.

use std::sync::Mutex;

use mira_vocabulary::{Event, EventKind, EventSeq, MiraError, Timestamp};

use super::sink::EventSink;

/// Collecte les événements émis, numérotés à partir de 1.
#[derive(Debug, Default)]
pub struct RecordingSink {
    events: Mutex<Vec<Event>>,
}

impl RecordingSink {
    /// Copie des événements émis jusqu'ici, dans l'ordre
    pub fn events(&self) -> Vec<Event> {
        self.events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    /// Copie des seuls genres d'événements émis, dans l'ordre
    pub fn kinds(&self) -> Vec<EventKind> {
        self.events().into_iter().map(|event| event.kind).collect()
    }
}

impl EventSink for RecordingSink {
    /// Enregistre l'événement avec le numéro suivant
    fn emit(&self, kind: EventKind) -> Result<Event, MiraError> {
        let mut events = self
            .events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let seq = EventSeq(u64::try_from(events.len()).unwrap_or(u64::MAX) + 1);
        let event = Event {
            seq,
            at: Timestamp::from_millis(0),
            kind,
        };
        events.push(event.clone());
        Ok(event)
    }
}
