//! Annulation de la dernière action confirmée par le moteur.

use mira_assembly::api::{ActionRequest, Request};

use crate::commands::{Command, Tag};
use crate::state::AppState;

/// Annule la dernière action confirmée ; sinon dit pourquoi dans la barre d'état.
pub fn undo(state: &mut AppState) -> Vec<Command> {
    if state.actions.in_flight() {
        state.status.message = Some("Action en cours, réessayez dans un instant".to_string());
        return Vec::new();
    }
    match state.actions.last_action.take() {
        Some(action) => vec![Command::Execute {
            tag: Tag::Undo(action.clone()),
            request: Request::Action(ActionRequest::UndoAction { action }),
        }],
        None => {
            state.status.message = Some("Rien à annuler".to_string());
            Vec::new()
        }
    }
}
