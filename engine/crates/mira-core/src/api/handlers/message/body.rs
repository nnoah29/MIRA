//! Lecture du corps d'un message déjà présent en local (S6 : sinon, erreur explicite).

use mira_vocabulary::{MessageId, MiraError};

use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::reading::get_body;

/// Rend le texte du message, ou `incompatible_state.body_unavailable_offline` s'il faut le télécharger
pub(crate) fn handle_get_body(engine: &Engine, message: &MessageId) -> Result<Response, MiraError> {
    Ok(Response::MessageBody(get_body(&engine.database, message)?))
}
