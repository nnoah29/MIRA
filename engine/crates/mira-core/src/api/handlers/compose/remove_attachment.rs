//! Retrait d'une pièce jointe d'un brouillon, d'après son rang.

use mira_vocabulary::{DraftId, MiraError};

use crate::api::contract::{Response, WriteResponse};
use crate::api::engine::Engine;
use crate::drafts::remove_attachment;

/// Retire la pièce jointe de ce rang (0 pour la première) et rend le brouillon à jour
pub(crate) fn handle_remove_attachment(
    engine: &Engine,
    draft: &DraftId,
    position: u32,
) -> Result<Response, MiraError> {
    let index = usize::try_from(position).unwrap_or(usize::MAX);
    let updated = remove_attachment(&engine.queue_context(), draft, index)?;
    Ok(Response::Write(WriteResponse::Draft(Box::new(updated))))
}
