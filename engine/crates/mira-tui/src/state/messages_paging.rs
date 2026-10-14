//! Suivi de la pagination de la liste de messages : vue, curseur suivant, demande en cours.

use mira_assembly::mira_vocabulary::{Cursor, ViewRef};

use super::messages::MessagesState;

impl MessagesState {
    /// Vrai si la liste est vide.
    pub fn is_empty(&self) -> bool {
        self.items().is_empty()
    }

    /// Vue dont les messages sont affichés.
    pub fn view(&self) -> Option<&ViewRef> {
        self.view.as_ref()
    }

    /// Curseur de la page suivante qui n'a pas encore été demandé.
    pub fn unrequested_cursor(&self) -> Option<&Cursor> {
        self.next_cursor
            .as_ref()
            .filter(|cursor| self.requested.as_ref() != Some(*cursor))
    }

    /// Note que la page suivante est demandée, pour ne pas la redemander.
    pub fn mark_requested(&mut self, cursor: Cursor) {
        self.requested = Some(cursor);
    }

    /// Autorise une nouvelle demande de la page suivante après un échec.
    pub fn release_request(&mut self) {
        self.requested = None;
    }
}
