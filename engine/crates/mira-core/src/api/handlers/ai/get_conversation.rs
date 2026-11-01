//! Lecture d'une conversation enregistrée avec son contenu.

use mira_vocabulary::MiraError;

use crate::api::contract::{AiResponse, Response};
use crate::api::engine::Engine;

/// Rend la conversation complète ; inconnue : `not_found.ai_conversation`
pub(crate) fn handle_get_ai_conversation(engine: &Engine, id: &str) -> Result<Response, MiraError> {
    let found = engine.database.read(|tx| tx.get_ai_conversation(id))?;
    Ok(Response::Ai(AiResponse::Conversation(found)))
}
