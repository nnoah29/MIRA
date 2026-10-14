//! Décision de l'utilisateur sur un envoi incertain : renvoyer ou abandonner (invariant 26).

use mira_vocabulary::{ActionId, MiraError};

use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::send::resolve_uncertain_send;

/// Renvoie (nouveau `Message-ID`) ou abandonne l'envoi incertain ; jamais de renvoi sans cette décision
pub(crate) fn handle_resolve_uncertain_send(
    engine: &Engine,
    action: &ActionId,
    resend: bool,
) -> Result<Response, MiraError> {
    resolve_uncertain_send(&engine.queue_context(), action, resend)?;
    engine.wake_replayers();
    Ok(Response::Done)
}
