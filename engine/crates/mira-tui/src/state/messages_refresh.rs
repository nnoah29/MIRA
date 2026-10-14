//! Première page d'une vue en cours de chargement : sert à ne pas redemander la même.

use super::messages::MessagesState;

impl MessagesState {
    /// Vrai tant que la première page de la vue voulue n'est pas arrivée.
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// Note qu'une première page est demandée.
    pub fn mark_loading(&mut self) {
        self.loading = true;
    }

    /// Note que la réponse de la première page est arrivée.
    pub fn finish_load(&mut self) {
        self.loading = false;
    }
}
