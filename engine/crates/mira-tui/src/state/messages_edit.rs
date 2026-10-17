//! Modifications locales (optimistes) d'un message de la liste.

use mira_assembly::mira_vocabulary::MessageId;

use super::messages::MessagesState;

impl MessagesState {
    /// Retire le message de la liste et recadre la sélection ; vrai s'il y était.
    pub fn remove(&mut self, id: &MessageId) -> bool {
        let before = self.items.len();
        self.items.retain(|item| &item.id != id);
        self.selection.clamp_to(self.items.len());
        self.items.len() != before
    }

    /// Fixe l'état « lu » du message dans la liste, sans effet s'il n'y est pas.
    pub fn set_seen(&mut self, id: &MessageId, seen: bool) {
        if let Some(item) = self.items.iter_mut().find(|item| &item.id == id) {
            item.flags.seen = seen;
        }
    }

    /// Fixe l'étoile du message dans la liste, sans effet s'il n'y est pas.
    pub fn set_flagged(&mut self, id: &MessageId, flagged: bool) {
        if let Some(item) = self.items.iter_mut().find(|item| &item.id == id) {
            item.flags.flagged = flagged;
        }
    }
}
