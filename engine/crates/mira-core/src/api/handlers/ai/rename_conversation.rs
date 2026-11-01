//! Renommage d'une conversation enregistrée ; son contenu et son rang dans la liste ne changent pas.

use mira_vocabulary::MiraError;

use super::conversation_input::validate_title;
use crate::api::contract::{AiResponse, Response};
use crate::api::engine::Engine;

/// Change le titre après validation ; inconnue : `not_found.ai_conversation`
pub(crate) fn handle_rename_ai_conversation(
    engine: &Engine,
    id: &str,
    title: &str,
) -> Result<Response, MiraError> {
    let title = validate_title(title)?;
    let meta = engine
        .database
        .write(|tx| tx.rename_ai_conversation(id, &title))?;
    Ok(Response::Ai(AiResponse::ConversationMeta(meta)))
}
