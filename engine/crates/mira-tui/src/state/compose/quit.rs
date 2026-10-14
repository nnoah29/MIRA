//! Sortie de l'application demandée alors qu'un brouillon modifié est ouvert.

use super::state::ComposeState;

impl ComposeState {
    /// Mémorise (ou oublie) que l'application doit quitter une fois le brouillon enregistré ou abandonné.
    pub fn set_then_quit(&mut self, quit: bool) {
        self.then_quit = quit;
    }

    /// Rend la demande de sortie différée et l'efface.
    pub fn take_then_quit(&mut self) -> bool {
        std::mem::take(&mut self.then_quit)
    }
}
