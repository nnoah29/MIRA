//! Déplacement dans la liste du panneau qui a le focus.

use crate::state::{AppState, Pane, Selection};

/// Applique un déplacement à la sélection du panneau actif ; le lecteur défile.
fn with_focused(
    state: &mut AppState,
    on_list: impl FnOnce(&mut Selection, usize),
    on_reader: impl FnOnce(u16) -> u16,
) {
    match state.focus {
        Pane::Sidebar => {
            let len = state.mailboxes.len();
            on_list(&mut state.mailboxes.selection, len);
        }
        Pane::List => {
            let len = state.messages.len();
            on_list(&mut state.messages.selection, len);
        }
        Pane::Reader => state.reader.scroll = on_reader(state.reader.scroll),
    }
}

/// Descend d'un élément.
pub fn move_down(state: &mut AppState) {
    with_focused(state, Selection::down, |scroll| scroll.saturating_add(1));
}

/// Monte d'un élément.
pub fn move_up(state: &mut AppState) {
    with_focused(state, Selection::up, |scroll| scroll.saturating_sub(1));
}

/// Va au début.
pub fn top(state: &mut AppState) {
    with_focused(state, Selection::top, |_| 0);
}

/// Va à la fin.
pub fn bottom(state: &mut AppState) {
    with_focused(state, Selection::bottom, |_| u16::MAX);
}
