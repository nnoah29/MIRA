//! Enregistrement d'une conversation avec l'assistant IA : création ou remplacement, puis rétention.

use mira_vocabulary::MiraError;

use super::conversation_input::{
    MAX_CONVERSATIONS, history_disabled, validate_payload, validate_title,
};
use crate::api::contract::{AiResponse, Response};
use crate::api::engine::Engine;

/// Valide puis enregistre la conversation et garde les 200 plus récentes, celle-ci comprise
pub(crate) fn handle_save_ai_conversation(
    engine: &Engine,
    id: &str,
    title: &str,
    payload_version: u32,
    payload: &str,
) -> Result<Response, MiraError> {
    let title = validate_title(title)?;
    validate_payload(payload)?;
    let now = engine.config.clock.now().as_millis();
    let meta = engine.database.write(|tx| {
        if !tx.ai_history_enabled()? {
            return Err(history_disabled());
        }
        let meta = tx.save_ai_conversation(id, &title, payload_version, payload, now)?;
        tx.trim_ai_conversations(MAX_CONVERSATIONS, id)?;
        Ok(meta)
    })?;
    Ok(Response::Ai(AiResponse::ConversationMeta(meta)))
}
