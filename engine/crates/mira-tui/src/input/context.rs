//! Contexte clavier : fenêtre ouverte, saisie de recherche ou panneau qui a le focus.

use crate::state::{AppState, Pane, Popup};

/// Ensemble de touches valables à un instant donné.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyContext {
    Sidebar,
    List,
    SearchResults,
    Reader,
    SearchInput,
    Confirm,
    MovePicker,
    Help,
}

/// Contexte du panneau qui a le focus, sans tenir compte d'une fenêtre ou d'une saisie ouverte.
pub fn pane_context(state: &AppState) -> KeyContext {
    match state.focus {
        Pane::Sidebar => KeyContext::Sidebar,
        Pane::List if state.search.is_active() => KeyContext::SearchResults,
        Pane::List => KeyContext::List,
        Pane::Reader => KeyContext::Reader,
    }
}

/// Contexte qui reçoit le clavier : la fenêtre ouverte, sinon la saisie, sinon le panneau.
pub fn context_of(state: &AppState) -> KeyContext {
    match &state.popup {
        Popup::ConfirmDelete { .. } => KeyContext::Confirm,
        Popup::MovePicker(_) => KeyContext::MovePicker,
        Popup::Help => KeyContext::Help,
        Popup::Closed if state.search.is_editing() => KeyContext::SearchInput,
        Popup::Closed => pane_context(state),
    }
}
