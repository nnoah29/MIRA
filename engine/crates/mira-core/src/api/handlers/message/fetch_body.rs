//! Téléchargement explicite du corps d'un message : opération longue et annulable.

use mira_vocabulary::{MessageId, MiraError};

use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::fetching::fetch_message_body;
use crate::operations::run_in_background;
use crate::reading::get_message;

/// Vérifie le message, démarre le téléchargement en arrière-plan et rend la fiche de l'opération
pub(crate) fn handle_fetch_message_body(
    engine: &Engine,
    message: MessageId,
) -> Result<Response, MiraError> {
    get_message(&engine.database, &message)?;
    let handle = engine.operations.start("fetch_message_body");
    let info = engine.operations.get(handle.id())?;
    let task = engine.fetch_task(handle.clone());
    run_in_background(
        handle,
        async move { fetch_message_body(&task, &message).await },
    );
    Ok(Response::Operation(info))
}
