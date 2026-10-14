//! Annulation d'un envoi pendant son délai : le brouillon revient en édition.

use mira_vocabulary::{ActionId, ActionKind, MiraError};

use crate::actions::cancel;
use crate::actions::errors::input::invalid_parameter;
use crate::actions::errors::not_found::action_not_found;
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Annule l'envoi en attente ; une action qui n'est pas un envoi est refusée
pub(crate) fn handle_cancel_send(
    engine: &Engine,
    action: &ActionId,
) -> Result<Response, MiraError> {
    let found = engine
        .database
        .read(|tx| tx.get_action(action))?
        .ok_or_else(|| action_not_found(action))?;
    if !matches!(found.kind, ActionKind::Send { .. }) {
        return Err(invalid_parameter("cette action n'est pas un envoi"));
    }
    cancel(&engine.queue_context(), action)?;
    Ok(Response::Done)
}
