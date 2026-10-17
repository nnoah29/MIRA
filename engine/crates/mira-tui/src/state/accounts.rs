//! Liste des comptes et sa sélection, toujours cohérentes entre elles.

use mira_assembly::mira_vocabulary::Account;

use super::selection::Selection;

/// Éléments affichés et sélection courante.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccountsState {
    items: Vec<Account>,
    pub selection: Selection,
}

impl AccountsState {
    /// Remplace les éléments et recadre la sélection sur la nouvelle longueur.
    pub fn replace_items(&mut self, items: Vec<Account>) {
        self.items = items;
        self.selection.clamp_to(self.items.len());
    }

    /// Éléments affichés.
    pub fn items(&self) -> &[Account] {
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

    /// Compte actuellement sélectionné.
    pub fn selected(&self) -> Option<&Account> {
        self.selection
            .selected
            .and_then(|index| self.items.get(index))
    }
}
