//! Réponses du moteur aux actions de l'utilisateur et à leur annulation.

use mira_assembly::api::{Response, WriteResponse};
use mira_assembly::mira_vocabulary::{IdempotencyKey, MiraError};

use super::accounts_reply::{report, unexpected};
use crate::commands::Command;
use crate::state::AppState;

/// Mémorise l'action confirmée pour l'annulation, si c'est bien la dernière envoyée.
pub fn on_user_action(
    state: &mut AppState,
    key: IdempotencyKey,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    let current = state.actions.awaits(&key);
    match result {
        Ok(Response::Write(WriteResponse::Action(action))) if current => {
            state.actions.confirm(action);
        }
        Ok(Response::Write(WriteResponse::Action(_))) => {}
        Ok(_) => return unexpected(state),
        Err(error) => {
            if current {
                state.actions.forget_awaited();
            }
            return report(state, error);
        }
    }
    Vec::new()
}

/// Confirme l'annulation dans la barre d'état, ou montre l'erreur.
pub fn on_undo(state: &mut AppState, result: Result<Response, MiraError>) -> Vec<Command> {
    match result {
        Ok(_) => {
            state.status.message = Some("Action annulée".to_string());
            Vec::new()
        }
        Err(error) => report(state, error),
    }
}
