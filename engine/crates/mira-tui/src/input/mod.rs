//! Touches du clavier → intentions, selon le contexte courant (spec §4.3) ; aucune I/O.

pub mod compose;
pub mod context;
pub mod global;
pub mod hints;
pub mod keys;
pub mod list;
pub mod popup;
pub mod reader;
pub mod search;
pub mod sidebar;

use crossterm::event::{KeyEvent, KeyEventKind};

pub use context::{KeyContext, context_of, pane_context};

use crate::state::AppState;
use crate::update::Intent;

/// Intention d'une touche dans l'état courant ; `None` pour une touche sans effet ici.
pub fn intent_for(state: &AppState, key: KeyEvent) -> Option<Intent> {
    if key.kind == KeyEventKind::Release {
        return None;
    }
    if keys::is_ctrl(key, 'c') {
        return Some(Intent::Quit);
    }
    match context_of(state) {
        KeyContext::SearchInput => search::intent(key),
        KeyContext::Creating => compose::creating_intent(key),
        KeyContext::Compose => compose::intent(key),
        KeyContext::ConfirmClose => popup::close_draft_intent(key),
        KeyContext::Confirm => popup::confirm_intent(key),
        KeyContext::MovePicker => popup::picker_intent(key),
        KeyContext::Help => popup::help_intent(key),
        KeyContext::Sidebar => in_pane(state, key, sidebar::intent),
        KeyContext::List | KeyContext::SearchResults => in_pane(state, key, list::intent),
        KeyContext::Reader => in_pane(state, key, reader::intent),
    }
}

/// Touches d'un panneau : d'abord les touches communes, puis celles propres au panneau.
fn in_pane(
    state: &AppState,
    key: KeyEvent,
    specific: fn(KeyEvent) -> Option<Intent>,
) -> Option<Intent> {
    global::intent(state, key).or_else(|| specific(key))
}
