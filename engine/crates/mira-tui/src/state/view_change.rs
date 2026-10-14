//! Changement de vue de la liste : le lecteur, qui montrait un message de l'ancienne vue, est vidé.

use mira_assembly::mira_vocabulary::ViewRef;

use super::{AppState, ReaderState};

impl AppState {
    /// Vide la liste et le lecteur, puis mémorise la vue dont le chargement va être demandé.
    pub fn begin_view_load(&mut self, view: ViewRef) {
        self.messages.begin_load(view);
        self.reader = ReaderState::default();
    }

    /// Vide la liste, oublie sa vue et vide le lecteur.
    pub fn clear_view(&mut self) {
        self.messages.reset();
        self.reader = ReaderState::default();
    }
}
