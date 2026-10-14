//! Réponse du moteur à la suppression d'un brouillon abandonné.

use mira_assembly::api::Response;
use mira_assembly::mira_vocabulary::{DraftId, MiraError};

use crate::commands::Command;
use crate::state::AppState;
use crate::update::accounts_reply::report;

/// Montre l'erreur d'une suppression échouée ; un succès ne change rien à l'écran.
pub fn on_draft_deleted(
    state: &mut AppState,
    _draft: DraftId,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    match result {
        Ok(_) => Vec::new(),
        Err(error) => report(state, error),
    }
}
