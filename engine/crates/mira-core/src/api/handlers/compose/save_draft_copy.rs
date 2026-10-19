//! Copie d'un brouillon dans la boîte Brouillons du serveur.

use mira_vocabulary::{DraftId, IdempotencyKey, MiraError};

use crate::api::contract::{Response, WriteResponse};
use crate::api::engine::Engine;
use crate::drafts::queue_draft_copy;

/// Met l'enregistrement serveur en file sans attendre le réseau ; la même clé rend la même action
pub(crate) fn handle_save_copy(
    engine: &Engine,
    draft: &DraftId,
    key: IdempotencyKey,
) -> Result<Response, MiraError> {
    let action = queue_draft_copy(&engine.queue_context(), draft, key)?;
    engine.wake_replayers();
    Ok(Response::Write(WriteResponse::Action(action)))
}
