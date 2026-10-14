//! Recherche : saisie en cours, requête active et vue à restaurer à la sortie.

use mira_assembly::mira_vocabulary::ViewRef;

/// Saisie ouverte ou non, texte tapé, requête dont les résultats sont affichés.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchState {
    pub(super) editing: bool,
    pub(super) input: String,
    pub(super) query: Option<String>,
    pub(super) previous: Option<ViewRef>,
    pub(super) first_page_only: Option<usize>,
}

impl SearchState {
    /// Vrai tant que la saisie de la requête est ouverte.
    pub fn is_editing(&self) -> bool {
        self.editing
    }

    /// Texte tapé dans la saisie.
    pub fn input(&self) -> &str {
        &self.input
    }

    /// Requête dont les résultats sont affichés, s'il y en a.
    pub fn query(&self) -> Option<&str> {
        self.query.as_deref()
    }

    /// Vrai si la liste affiche des résultats de recherche.
    pub fn is_active(&self) -> bool {
        self.query.is_some()
    }
}
