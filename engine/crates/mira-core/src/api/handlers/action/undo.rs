//! Annulation d'une action récente, tant qu'elle est dans son délai d'annulation.

use mira_vocabulary::{ActionId, MiraError};

use crate::actions::cancel;
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Annule l'action en attente et défait son effet local ; sinon `action_not_cancellable`
pub(crate) fn handle_undo_action(
    engine: &Engine,
    action: &ActionId,
) -> Result<Response, MiraError> {
    cancel(&engine.queue_context(), action)?;
    Ok(Response::Done)
}
