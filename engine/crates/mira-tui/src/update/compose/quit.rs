//! Sortie différée : quitter avec un brouillon modifié passe par la confirmation de fermeture.

use crate::commands::Command;
use crate::state::{AppState, Popup};
use crate::update::panes;

/// Ouvre la confirmation de fermeture et mémorise la sortie si le brouillon a des modifications ; vrai alors.
pub fn defer_quit(state: &mut AppState) -> bool {
    let unsaved = state.compose.open().is_some_and(|c| c.has_unsaved());
    if unsaved {
        mark_then_quit(state, true);
        state.popup = Popup::ConfirmCloseDraft;
    }
    unsaved
}

/// Note si la sortie doit suivre la fermeture du brouillon.
pub fn mark_then_quit(state: &mut AppState, quit: bool) {
    if let Some(compose) = state.compose.open_mut() {
        compose.set_then_quit(quit);
    }
}

/// Rend la demande de sortie différée du brouillon ouvert et l'efface.
pub fn take_then_quit(state: &mut AppState) -> bool {
    state.compose.open_mut().is_some_and(|c| c.take_then_quit())
}

/// Ajoute la sortie aux commandes de fermeture si elle était demandée.
pub fn with_quit(state: &mut AppState, quit: bool, mut commands: Vec<Command>) -> Vec<Command> {
    if quit {
        commands.extend(panes::quit(state));
    }
    commands
}
