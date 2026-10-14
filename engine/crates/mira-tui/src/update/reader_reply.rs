//! Réponses du moteur qui alimentent le lecteur ; celles d'un autre message sont ignorées.

use mira_assembly::api::Response;
use mira_assembly::mira_vocabulary::error::codes::incompatible_state::BODY_UNAVAILABLE_OFFLINE;
use mira_assembly::mira_vocabulary::{Attachment, MessageId, MiraError, ThreadId};

use super::accounts_reply::{report, unexpected};
use crate::commands::Command;
use crate::state::{AppState, ReaderContent};
use crate::view::safe_text::display_safe_multiline;

/// Range le corps du message ouvert ; l'absence hors ligne n'est pas une erreur.
pub fn on_body(
    state: &mut AppState,
    id: MessageId,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    if !state.reader.shows(&id) {
        return Vec::new();
    }
    match result {
        Ok(Response::MessageBody(mut body)) => {
            body.text = display_safe_multiline(&body.text);
            state.reader.content = ReaderContent::Body(body);
        }
        Ok(_) => return unexpected(state),
        Err(error) if is_offline_absence(&error) => {
            state.reader.content = ReaderContent::OfflineMissing;
        }
        Err(error) => {
            state.reader.content = ReaderContent::Failed(display_safe_multiline(&error.message));
            return report(state, error);
        }
    }
    Vec::new()
}

/// Vrai pour l'erreur « corps indisponible hors ligne ».
fn is_offline_absence(error: &MiraError) -> bool {
    error.code == BODY_UNAVAILABLE_OFFLINE.0
}

/// Range les pièces jointes du message ouvert.
pub fn on_attachments(
    state: &mut AppState,
    id: MessageId,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    if !state.reader.shows(&id) {
        return Vec::new();
    }
    match result {
        Ok(Response::Attachments(files)) => store_attachments(state, files),
        Ok(_) => return unexpected(state),
        Err(error) => return report(state, error),
    }
    Vec::new()
}

/// Remplace les pièces jointes affichées.
fn store_attachments(state: &mut AppState, files: Vec<Attachment>) {
    state.reader.attachments = files;
}

/// Range le fil du message ouvert s'il en fait partie.
pub fn on_thread(
    state: &mut AppState,
    thread: ThreadId,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    let current = state.reader.message().and_then(|m| m.thread.as_ref());
    if current != Some(&thread) {
        return Vec::new();
    }
    match result {
        Ok(Response::Thread(view)) => state.reader.thread = view.messages,
        Ok(_) => return unexpected(state),
        Err(error) => return report(state, error),
    }
    Vec::new()
}

/// Traite la réponse d'une action : seule une erreur est à montrer.
pub fn on_action(state: &mut AppState, result: Result<Response, MiraError>) -> Vec<Command> {
    match result {
        Ok(_) => Vec::new(),
        Err(error) => report(state, error),
    }
}
