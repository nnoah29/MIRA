//! Réponse du moteur à l'envoi ; seule celle du brouillon ouvert, de sa clé et d'un envoi en vol compte.

use mira_assembly::api::{Response, WriteResponse};
use mira_assembly::mira_vocabulary::{DraftId, IdempotencyKey, MiraError};

use super::accounts_reply::unexpected;
use crate::commands::Command;
use crate::state::compose::SendPhase;
use crate::state::{AppState, ComposeSlot};
use crate::view::safe_text::display_safe;

/// Envoi pris en file : l'éditeur se ferme (le brouillon appartient à l'envoi) ; échec : texte gardé, même clé au renvoi.
pub fn on_sent(
    state: &mut AppState,
    draft: DraftId,
    key: IdempotencyKey,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    if !awaits(state, &draft, &key) {
        return Vec::new();
    }
    match result {
        Ok(Response::Write(WriteResponse::Action(_))) => {
            state.compose = ComposeSlot::Closed;
            state.status.message =
                Some("En file d'envoi : annulable un court instant depuis la file (Q)".to_string());
            Vec::new()
        }
        Ok(_) => {
            back_to_idle(state);
            unexpected(state)
        }
        Err(error) => {
            back_to_idle(state);
            let reason = display_safe(&error.to_string());
            state.status.message = Some(format!("Échec de l'envoi : {reason}"));
            Vec::new()
        }
    }
}

/// Vrai si cette réponse est celle de l'envoi en vol du brouillon ouvert.
fn awaits(state: &AppState, draft: &DraftId, key: &IdempotencyKey) -> bool {
    state.compose.open().is_some_and(|compose| {
        compose.draft() == draft
            && compose.send_phase() == SendPhase::Sending
            && compose.send_key.as_ref() == Some(key)
    })
}

/// L'envoi n'est plus en vol : l'éditeur redevient modifiable.
fn back_to_idle(state: &mut AppState) {
    if let Some(compose) = state.compose.open_mut() {
        compose.set_send_phase(SendPhase::Idle);
    }
}
