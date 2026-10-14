//! Changement de panneau et sortie de l'application.

use super::message_load::load_selected_mailbox;
use super::reader_open::{open_selected_message, retry_body_download};
use crate::commands::Command;
use crate::state::{AppState, Pane};

/// Ouvre l'élément sélectionné en passant au panneau de droite ; une boîte ouverte est chargée.
pub fn open(state: &mut AppState) -> Vec<Command> {
    match state.focus {
        Pane::Sidebar => {
            state.focus = Pane::List;
            load_selected_mailbox(state)
        }
        Pane::List => {
            state.focus = Pane::Reader;
            open_selected_message(state)
        }
        Pane::Reader => retry_body_download(state),
    }
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
