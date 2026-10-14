//! Retour à la vue d'avant la recherche.

use mira_assembly::mira_vocabulary::ViewRef;

use super::search::SearchState;

impl SearchState {
    /// Rend la vue à restaurer, une seule fois.
    pub fn take_previous(&mut self) -> Option<ViewRef> {
        self.previous.take()
    }

    /// Abandonne tout : saisie, requête et vue mémorisée (changement de boîte ou de compte).
    pub fn cancel(&mut self) {
        self.leave();
        self.previous = None;
    }
}
