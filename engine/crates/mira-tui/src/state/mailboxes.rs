//! Liste des boîtes et sa sélection, toujours cohérentes entre elles.

use mira_assembly::mira_vocabulary::Mailbox;

use super::selection::Selection;

/// Éléments affichés et sélection courante.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MailboxesState {
    items: Vec<Mailbox>,
    pub selection: Selection,
}

impl MailboxesState {
    /// Remplace les éléments et recadre la sélection sur la nouvelle longueur.
    pub fn replace_items(&mut self, items: Vec<Mailbox>) {
        self.items = items;
        self.selection.clamp_to(self.items.len());
    }

    /// Boîte actuellement sélectionnée.
    pub fn selected(&self) -> Option<&Mailbox> {
        self.selection
            .selected
            .and_then(|index| self.items.get(index))
    }

    /// Éléments affichés.
    pub fn items(&self) -> &[Mailbox] {
        &self.items
    }

    /// Nombre d'éléments.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Vrai si la liste est vide.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
