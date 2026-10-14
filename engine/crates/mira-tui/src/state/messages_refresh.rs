//! Première page d'une vue en cours de chargement : sert à ne pas redemander la même.

use mira_assembly::mira_vocabulary::MessageId;

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

    /// Identifiant du message sélectionné.
    pub fn selected_id(&self) -> Option<MessageId> {
        let index = self.selection.selected?;
        self.items.get(index).map(|item| item.id.clone())
    }

    /// Replace la sélection sur ce message s'il est encore dans la liste, sinon garde le recadrage par indice.
    pub fn reanchor(&mut self, anchor: Option<MessageId>) {
        let found = anchor.and_then(|id| self.items.iter().position(|item| item.id == id));
        if found.is_some() {
            self.selection.selected = found;
        }
    }
}
