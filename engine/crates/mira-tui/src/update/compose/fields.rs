//! Changement de champ : Tab et Maj+Tab.

use crate::commands::Command;
use crate::state::AppState;

/// Passe au champ suivant.
pub fn next_field(state: &mut AppState) -> Vec<Command> {
    if let Some(compose) = state.compose.open_mut() {
        compose.focus_next();
    }
    Vec::new()
}

/// Passe au champ précédent.
pub fn previous_field(state: &mut AppState) -> Vec<Command> {
    if let Some(compose) = state.compose.open_mut() {
        compose.focus_previous();
    }
    Vec::new()
}
