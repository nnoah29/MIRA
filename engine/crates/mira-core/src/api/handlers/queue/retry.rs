//! Relance d'une action échouée.

use mira_vocabulary::{ActionId, MiraError};

use crate::actions::retry;
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Remet l'action échouée en file, effet local réappliqué, et réveille le rejeu
pub(crate) fn handle_retry_action(
    engine: &Engine,
    action: &ActionId,
) -> Result<Response, MiraError> {
    retry(&engine.queue_context(), action)?;
    engine.wake_replayers();
    Ok(Response::Done)
}
