//! Journal ordonné des effets observés sur le terminal factice.

use std::sync::{Arc, Mutex};

/// Liste partagée des événements, dans l'ordre où ils se produisent.
#[derive(Debug, Clone, Default)]
pub struct EventLog {
    events: Arc<Mutex<Vec<String>>>,
}

impl EventLog {
    /// Ajoute un événement à la suite des précédents.
    pub fn push(&self, event: &str) {
        if let Ok(mut events) = self.events.lock() {
            events.push(event.to_string());
        }
    }

    /// Copie les événements notés jusqu'ici.
    pub fn snapshot(&self) -> Vec<String> {
        self.events
            .lock()
            .map(|events| events.clone())
            .unwrap_or_default()
    }
}
