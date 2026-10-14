//! Annulation de la dernière action confirmée par le moteur.

use mira_assembly::api::{ActionRequest, Request};

use crate::commands::{Command, Tag};
use crate::state::AppState;

/// Annule la dernière action mémorisée, ou le dit dans la barre d'état s'il n'y en a pas.
pub fn undo(state: &mut AppState) -> Vec<Command> {
    match state.actions.last_action.take() {
        Some(action) => vec![Command::Execute {
            tag: Tag::Undo,
            request: Request::Action(ActionRequest::UndoAction { action }),
        }],
        None => {
            state.status.message = Some("Rien à annuler".to_string());
            Vec::new()
        }
    }
}
