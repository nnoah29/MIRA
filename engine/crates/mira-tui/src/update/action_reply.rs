//! Réponses du moteur aux actions de l'utilisateur et à leur annulation.

use mira_assembly::api::{Response, WriteResponse};
use mira_assembly::mira_vocabulary::{ActionId, IdempotencyKey, MiraError};

use super::accounts_reply::{report, unexpected};
use super::message_load::load_view;
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
        Err(error) => return fail_action(state, current, error),
    }
    Vec::new()
}

/// Montre l'erreur ; si c'est la dernière action, recharge la vue pour défaire le changement optimiste.
fn fail_action(state: &mut AppState, current: bool, error: MiraError) -> Vec<Command> {
    let mut commands = report(state, error);
    if !current {
        return commands;
    }
    state.actions.forget_awaited();
    if let Some(view) = state.messages.view().cloned() {
        commands.extend(load_view(state, view));
    }
    commands
}

/// Confirme l'annulation ; à l'échec, rend l'action pour pouvoir réessayer.
pub fn on_undo(
    state: &mut AppState,
    action: ActionId,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    match result {
        Ok(_) => {
            state.status.message = Some("Action annulée".to_string());
            Vec::new()
        }
        Err(error) => {
            if !state.actions.in_flight() && state.actions.last_action.is_none() {
                state.actions.last_action = Some(action);
            }
            report(state, error)
        }
    }
}
