//! Changement de vue de la liste : le lecteur, qui montrait un message de l'ancienne vue, est vidé.

use super::{AppState, ReaderState};

impl AppState {
    /// Vide la liste, oublie sa vue et vide le lecteur.
    pub fn clear_view(&mut self) {
        self.messages.reset();
        self.reader = ReaderState::default();
    }
}
