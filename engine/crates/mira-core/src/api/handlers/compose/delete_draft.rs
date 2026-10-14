//! Suppression d'un brouillon en cours d'édition.

use mira_vocabulary::{DraftId, MiraError};

use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::drafts::delete_draft;

/// Supprime le brouillon et ses pièces jointes
pub(crate) fn handle_delete_draft(engine: &Engine, draft: &DraftId) -> Result<Response, MiraError> {
    delete_draft(&engine.queue_context(), draft)?;
    Ok(Response::Done)
}
