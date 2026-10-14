//! Lecture des en-têtes d'un message (lecture locale).

use mira_vocabulary::{MessageId, MiraError};

use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::reading::get_message;

/// Rend le résumé du message, ou `not_found.message`
pub(crate) fn handle_get_message(
    engine: &Engine,
    message: &MessageId,
) -> Result<Response, MiraError> {
    Ok(Response::Message(get_message(&engine.database, message)?))
}
