//! Liste des pièces jointes d'un message (lecture locale).

use mira_vocabulary::{MessageId, MiraError};

use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::reading::list_attachments;

/// Rend les pièces jointes du message, dans l'ordre des parties
pub(crate) fn handle_list_attachments(
    engine: &Engine,
    message: &MessageId,
) -> Result<Response, MiraError> {
    Ok(Response::Attachments(list_attachments(
        &engine.database,
        message,
    )?))
}
