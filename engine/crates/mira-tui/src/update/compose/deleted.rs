//! Réponse du moteur à la suppression d'un brouillon (abandonné, ou demandé depuis la vue « Brouillons »).

use mira_assembly::api::Response;
use mira_assembly::mira_vocabulary::{DraftId, MiraError};

use crate::commands::Command;
use crate::state::AppState;
use crate::update::accounts_reply::report;
use crate::update::drafts::delete::on_deleted;

/// Suppression demandée par la vue « Brouillons » : traitée par elle ; sinon l'erreur d'une suppression échouée est montrée.
pub fn on_draft_deleted(
    state: &mut AppState,
    draft: DraftId,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    if let Some(subject) = state.drafts.take_delete(&draft) {
        return on_deleted(state, &subject, result);
    }
    match result {
        Ok(_) => Vec::new(),
        Err(error) => report(state, error),
    }
}
