//! Liste des comptes et sa sélection, toujours cohérentes entre elles.

use super::selection::Selection;

/// Éléments affichés et sélection courante.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccountsState {
    items: Vec<String>,
    pub selection: Selection,
}

impl AccountsState {
    /// Remplace les éléments et recadre la sélection sur la nouvelle longueur.
    pub fn replace_items(&mut self, items: Vec<String>) {
        self.items = items;
        self.selection.clamp_to(self.items.len());
    }

    /// Éléments affichés.
    pub fn items(&self) -> &[String] {
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
