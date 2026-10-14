//! Annulation d'une action de la file tant qu'elle est en attente.

use mira_vocabulary::{ActionId, MiraError};

use crate::actions::cancel;
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Annule l'action en attente et défait son effet local
pub(crate) fn handle_cancel_action(
    engine: &Engine,
    action: &ActionId,
) -> Result<Response, MiraError> {
    cancel(&engine.queue_context(), action)?;
    Ok(Response::Done)
}
