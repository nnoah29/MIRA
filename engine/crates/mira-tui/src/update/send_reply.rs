//! Réponse du moteur à l'envoi ; seule celle du brouillon ouvert, de sa clé et d'un envoi en vol compte.

use mira_assembly::api::{Response, WriteResponse};
use mira_assembly::mira_vocabulary::{ActionId, DraftId, IdempotencyKey, MiraError};

use super::accounts_reply::unexpected;
use crate::commands::Command;
use crate::state::compose::{Field, SendPhase};
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
        Ok(Response::Write(WriteResponse::Action(action))) => accept(state, action),
        Ok(_) => {
            back_to_idle(state);
            unexpected(state)
        }
        Err(error) => fail(state, &error),
    }
}

/// Envoi en file : l'éditeur se ferme et l'objet est retenu pour annoncer le départ.
fn accept(state: &mut AppState, action: ActionId) -> Vec<Command> {
    if let ComposeSlot::Open(compose) = std::mem::take(&mut state.compose) {
        let subject = compose.text(Field::Subject);
        state.sent_subjects.remember(action, subject);
    }
    state.status.message =
        Some("En file d'envoi : annulable un court instant depuis la file (Q)".to_string());
    Vec::new()
}

/// Échec : l'éditeur redevient modifiable, l'échec est retenu et le renvoi sûr est indiqué.
fn fail(state: &mut AppState, error: &MiraError) -> Vec<Command> {
    back_to_idle(state);
    if let Some(compose) = state.compose.open_mut() {
        compose.set_send_failed(true);
    }
    let reason = display_safe(&error.to_string());
    state.status.message = Some(format!(
        "Échec de l'envoi : {reason} — F5 réessaie le même envoi, sans doublon"
    ));
    Vec::new()
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
