//! Liste des messages et sa sélection, toujours cohérentes entre elles.

use mira_assembly::mira_vocabulary::{Cursor, MessageSummary, Page, ViewRef};

use super::selection::Selection;

/// Éléments affichés, vue d'origine, pagination et sélection courante.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MessagesState {
    pub(super) items: Vec<MessageSummary>,
    pub(super) view: Option<ViewRef>,
    pub(super) next_cursor: Option<Cursor>,
    pub(super) requested: Option<Cursor>,
    pub(super) loading: bool,
    pub selection: Selection,
}

impl MessagesState {
    /// Remplace les éléments, oublie la pagination et recadre la sélection.
    pub fn replace_items(&mut self, items: Vec<MessageSummary>) {
        self.items = items;
        self.next_cursor = None;
        self.requested = None;
        self.selection.clamp_to(self.items.len());
    }

    /// Ajoute des éléments à la suite et recadre la sélection.
    pub fn append_items(&mut self, items: Vec<MessageSummary>) {
        self.items.extend(items);
        self.selection.clamp_to(self.items.len());
    }

    /// Remplace la liste par la première page d'une vue ; changer de vue repart du premier message.
    pub fn replace_page(&mut self, view: ViewRef, page: Page<MessageSummary>) {
        if self.view.as_ref() != Some(&view) {
            self.selection.selected = None;
        }
        self.view = Some(view);
        self.loading = false;
        self.replace_items(page.items);
        self.next_cursor = page.next_cursor;
    }

    /// Ajoute la page suivante et mémorise le curseur de la page d'après.
    pub fn append_page(&mut self, page: Page<MessageSummary>) {
        self.append_items(page.items);
        self.next_cursor = page.next_cursor;
        self.requested = None;
    }

    /// Éléments affichés.
    pub fn items(&self) -> &[MessageSummary] {
        &self.items
    }

    /// Nombre d'éléments.
    pub fn len(&self) -> usize {
        self.items.len()
    }
}
