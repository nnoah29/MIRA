//! Contexte clavier : fenêtre ouverte, éditeur de brouillon, saisie de recherche ou panneau qui a le focus.

use super::account_form::form_context;
use crate::state::{AppState, Pane, Popup};

/// Ensemble de touches valables à un instant donné.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyContext {
    Sidebar,
    List,
    SearchResults,
    Reader,
    SearchInput,
    Creating,
    Compose,
    ConfirmClose,
    ConfirmDiscard,
    ConfirmSend,
    ResolveUncertain,
    Queue,
    Drafts,
    ConfirmDeleteDraft,
    Confirm,
    MovePicker,
    Help,
    AccountForm,
    AccountFormConfirm,
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

/// Contexte qui reçoit le clavier : la fenêtre ouverte, sinon l'éditeur, la vue « file » ou « Brouillons », la saisie, le panneau.
pub fn context_of(state: &AppState) -> KeyContext {
    if let Some(context) = form_context(state) {
        return context;
    }
    match &state.popup {
        Popup::ConfirmDelete { .. } => KeyContext::Confirm,
        Popup::MovePicker(_) => KeyContext::MovePicker,
        Popup::Help => KeyContext::Help,
        Popup::ConfirmCloseDraft => KeyContext::ConfirmClose,
        Popup::ConfirmDiscardDraft => KeyContext::ConfirmDiscard,
        Popup::ConfirmSend => KeyContext::ConfirmSend,
        Popup::ResolveUncertain { .. } => KeyContext::ResolveUncertain,
        Popup::ConfirmDeleteDraft { .. } => KeyContext::ConfirmDeleteDraft,
        Popup::Closed if state.compose.is_creating() => KeyContext::Creating,
        Popup::Closed if state.compose.open().is_some() => KeyContext::Compose,
        Popup::Closed if state.queue.is_open() => KeyContext::Queue,
        Popup::Closed if state.drafts.is_open() => KeyContext::Drafts,
        Popup::Closed if state.search.is_editing() => KeyContext::SearchInput,
        Popup::Closed => pane_context(state),
    }
}
