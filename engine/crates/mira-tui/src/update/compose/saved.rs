//! Réponse du moteur à l'enregistrement ; celle d'un brouillon qui n'est plus ouvert est ignorée.

use mira_assembly::api::{Response, WriteResponse};
use mira_assembly::mira_vocabulary::{DraftId, MiraError};

use super::close::continue_close;
use crate::commands::Command;
use crate::state::AppState;
use crate::update::accounts_reply::{report, unexpected};

/// Prend acte de l'enregistrement du brouillon ouvert : succès, ou échec qui laisse tout à enregistrer.
pub fn on_draft_saved(
    state: &mut AppState,
    draft: DraftId,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    let Some(compose) = state.compose.open_mut() else {
        return Vec::new();
    };
    if compose.draft() != &draft || !compose.is_saving() {
        return Vec::new();
    }
    match result {
        Ok(Response::Write(WriteResponse::Draft(_))) => {
            compose.finish_save();
            state.status.message = Some("Brouillon enregistré".to_string());
            continue_close(state)
        }
        Ok(_) => {
            compose.fail_save();
            unexpected(state)
        }
        Err(error) => {
            compose.fail_save();
            report(state, error)
        }
    }
}
