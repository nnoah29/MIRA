//! Retour à la vue d'avant la recherche et troncature des résultats affichés.

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

    /// Note le nombre de résultats reçus quand le moteur en a d'autres (seule la première page est montrée).
    pub fn note_results(&mut self, count: usize, has_more: bool) {
        self.first_page_only = has_more.then_some(count);
    }

    /// Nombre de résultats affichés si la liste est tronquée à la première page.
    pub fn first_page_only(&self) -> Option<usize> {
        self.first_page_only
    }
}
