//! Changement de panneau et sortie de l'application.

use crate::commands::Command;
use crate::state::{AppState, Pane};

/// Ouvre l'élément sélectionné en passant au panneau de droite.
pub fn open(state: &mut AppState) {
    state.focus = match state.focus {
        Pane::Sidebar => Pane::List,
        Pane::List | Pane::Reader => Pane::Reader,
    };
}

/// Revient au panneau de gauche.
pub fn back(state: &mut AppState) {
    state.focus = match state.focus {
        Pane::Reader => Pane::List,
        Pane::List | Pane::Sidebar => Pane::Sidebar,
    };
}

/// Passe au panneau suivant.
pub fn next_pane(state: &mut AppState) {
    state.focus = state.focus.next();
}

/// Passe au panneau précédent.
pub fn prev_pane(state: &mut AppState) {
    state.focus = state.focus.previous();
}

/// Demande la sortie de l'application.
pub fn quit(state: &mut AppState) -> Vec<Command> {
    state.should_quit = true;
    vec![Command::Quit]
}
