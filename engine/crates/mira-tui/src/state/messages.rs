//! Liste des messages et sa sélection.

use super::selection::Selection;

/// Éléments affichés et sélection courante.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MessagesState {
    pub items: Vec<String>,
    pub selection: Selection,
}
