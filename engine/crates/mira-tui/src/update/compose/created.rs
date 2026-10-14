//! Réponse du moteur à la création d'un brouillon ; celle d'une autre demande est ignorée.

use mira_assembly::api::{ComposeRequest, Request, Response, WriteResponse};
use mira_assembly::mira_vocabulary::{Draft, IdempotencyKey, MiraError};

use crate::commands::{Command, Tag};
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
        return delete_orphan(state, &token, result);
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

/// Annule la création en attente ; sa réponse, si elle arrive, sera ignorée et son brouillon supprimé.
pub fn cancel_creation(state: &mut AppState) -> Vec<Command> {
    if let ComposeSlot::Creating(token) = std::mem::take(&mut state.compose) {
        state.cancelled_drafts.push(token);
        state.status.message = Some("Création du brouillon annulée".to_string());
    }
    Vec::new()
}

/// Réponse d'une création annulée : le brouillon que le moteur a quand même créé est supprimé.
fn delete_orphan(
    state: &mut AppState,
    token: &IdempotencyKey,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    let Some(position) = state.cancelled_drafts.iter().position(|t| t == token) else {
        return Vec::new();
    };
    state.cancelled_drafts.remove(position);
    match result {
        Ok(Response::Write(WriteResponse::Draft(draft))) => vec![Command::Execute {
            tag: Tag::DraftDeleted(draft.id.clone()),
            request: Request::Compose(ComposeRequest::DeleteDraft { draft: draft.id }),
        }],
        Ok(_) | Err(_) => Vec::new(),
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
