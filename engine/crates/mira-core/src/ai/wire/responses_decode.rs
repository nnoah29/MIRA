//! Décodage d'un événement SSE au format Responses en deltas, sans copier de contenu d'erreur.

use mira_vocabulary::{AiError, ChatDelta};
use serde_json::Value;

use super::responses_finish::{completed_reason, finish, incomplete_reason};
use super::responses_tool_decode::{added_fragment, arguments_fragment};
use crate::ai::sse::SseEvent;

const LIMIT_EXCEEDED: &str = "subscription_sharing_usage_limit_exceeded";
const USAGE_UNAVAILABLE: &str = "subscription_sharing_usage_unavailable";

/// Décode un événement selon son `type` ; ceux qu'on ne connaît pas ne produisent rien.
pub fn decode_chat_event(event: &SseEvent) -> Result<Vec<ChatDelta>, AiError> {
    let data = event.data.trim();
    if data.is_empty() {
        return Ok(Vec::new());
    }
    let chunk: Value = serde_json::from_str(data).map_err(|_| AiError::BadResponse)?;
    match chunk.get("type").and_then(Value::as_str) {
        Some("response.output_text.delta") => Ok(text_delta(&chunk).into_iter().collect()),
        Some("response.output_item.added") => Ok(added_fragment(&chunk).into_iter().collect()),
        Some("response.function_call_arguments.delta") => {
            Ok(arguments_fragment(&chunk).into_iter().collect())
        }
        Some("response.completed") => Ok(finish(&chunk, completed_reason(&chunk))),
        Some("response.incomplete") => Ok(finish(&chunk, incomplete_reason(&chunk))),
        Some("response.failed") => Err(failure(&chunk)),
        _ => Ok(Vec::new()),
    }
}

/// Texte non vide d'un morceau de sortie.
fn text_delta(chunk: &Value) -> Option<ChatDelta> {
    let text = chunk.get("delta")?.as_str()?;
    (!text.is_empty()).then(|| ChatDelta::Text {
        text: text.to_string(),
    })
}

/// Erreur selon le code d'échec, sans recopier le message du service.
fn failure(chunk: &Value) -> AiError {
    match chunk
        .pointer("/response/error/code")
        .and_then(Value::as_str)
    {
        Some(LIMIT_EXCEEDED) => AiError::QuotaExhausted,
        Some(USAGE_UNAVAILABLE) => AiError::Unavailable,
        _ => AiError::BadResponse,
    }
}
