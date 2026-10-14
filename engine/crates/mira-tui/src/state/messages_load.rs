//! Chargement d'une vue de messages : vue voulue, remise à zéro et demande en attente.

use mira_assembly::mira_vocabulary::{Cursor, ViewRef};

use super::messages::MessagesState;

impl MessagesState {
    /// Vide la liste et mémorise la vue dont le chargement va être demandé.
    pub fn begin_load(&mut self, view: ViewRef) {
        self.reset();
        self.view = Some(view);
    }

    /// Vide la liste et oublie la vue voulue.
    pub fn reset(&mut self) {
        self.replace_items(Vec::new());
        self.view = None;
    }

    /// Vrai si la page après ce curseur est bien celle qui est attendue pour cette vue.
    pub fn awaits(&self, view: &ViewRef, cursor: &Cursor) -> bool {
        self.view.as_ref() == Some(view) && self.requested.as_ref() == Some(cursor)
    }
}
