//! Envoi d'un brouillon, tout de suite (après le délai d'annulation) ou à une date choisie.

use mira_vocabulary::{DraftId, IdempotencyKey, MiraError, Timestamp};

use crate::api::contract::{Response, WriteResponse};
use crate::api::engine::Engine;
use crate::drafts::schedule_send;

/// Planifie l'envoi et rend l'action sans attendre le réseau ; la même clé rend la même action
pub(crate) fn handle_send_draft(
    engine: &Engine,
    draft: &DraftId,
    at: Option<Timestamp>,
    key: IdempotencyKey,
) -> Result<Response, MiraError> {
    let action = schedule_send(&engine.queue_context(), draft, at, key)?;
    engine.wake_replayers();
    Ok(Response::Write(WriteResponse::Action(action)))
}
