//! Réponse du moteur à la création d'un brouillon ; celle d'une autre demande est ignorée.

use mira_assembly::api::{Response, WriteResponse};
use mira_assembly::mira_vocabulary::{Draft, IdempotencyKey, MiraError};

use crate::commands::Command;
use crate::state::compose::ComposeState;
use crate::state::{AppState, ComposeSlot, Popup};
use crate::update::accounts_reply::{report, unexpected};

/// Ouvre l'éditeur sur le brouillon reçu si c'est bien celui qui est attendu.
pub fn on_draft_created(
    state: &mut AppState,
    token: IdempotencyKey,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    if !state.compose.awaits(&token) {
        return Vec::new();
    }
    match result {
        Ok(Response::Write(WriteResponse::Draft(draft))) => open_editor(state, &draft),
        Ok(_) => {
            state.compose = ComposeSlot::Closed;
            unexpected(state)
        }
        Err(error) => {
            state.compose = ComposeSlot::Closed;
            report(state, error)
        }
    }
}

/// Ouvre l'éditeur ; une fenêtre ou une saisie de recherche ouverte entre-temps est fermée.
fn open_editor(state: &mut AppState, draft: &Draft) -> Vec<Command> {
    state.compose = ComposeSlot::Open(Box::new(ComposeState::from_draft(draft)));
    state.popup = Popup::Closed;
    state.search.close_input();
    state.status.message = None;
    Vec::new()
}
