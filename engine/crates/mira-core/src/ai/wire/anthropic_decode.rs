//! Décodage d'un événement SSE au format Messages en deltas, sans copier de contenu d'erreur.

use mira_vocabulary::{AiError, ChatDelta, StopReason};
use serde_json::Value;

use super::anthropic_usage::usage_delta;
use crate::ai::sse::SseEvent;

/// Décode un événement selon le type porté par ses données ; les types inconnus sont ignorés.
pub fn decode_chat_event(event: &SseEvent) -> Result<Vec<ChatDelta>, AiError> {
    let data = event.data.trim();
    if data.is_empty() {
        return Ok(Vec::new());
    }
    let payload: Value = serde_json::from_str(data).map_err(|_| AiError::BadResponse)?;
    match payload.get("type").and_then(Value::as_str) {
        Some("content_block_delta") => Ok(text_delta(&payload).into_iter().collect()),
        Some("message_start") => Ok(usage_delta(payload.pointer("/message/usage"))
            .into_iter()
            .collect()),
        Some("message_delta") => Ok(message_end(&payload)),
        Some("message_stop") => Ok(vec![done(StopReason::EndTurn)]),
        Some("error") => Err(AiError::Unavailable),
        _ => Ok(Vec::new()),
    }
}

/// Extrait le texte non vide d'un delta de type texte.
fn text_delta(payload: &Value) -> Option<ChatDelta> {
    let delta = payload.get("delta")?;
    if delta.get("type")?.as_str()? != "text_delta" {
        return None;
    }
    let text = delta.get("text")?.as_str()?;
    (!text.is_empty()).then(|| ChatDelta::Text {
        text: text.to_string(),
    })
}

/// Lit la consommation puis la raison d'arrêt éventuelle d'un message_delta.
fn message_end(payload: &Value) -> Vec<ChatDelta> {
    let mut deltas: Vec<ChatDelta> = usage_delta(payload.get("usage")).into_iter().collect();
    let reason = payload
        .pointer("/delta/stop_reason")
        .and_then(Value::as_str);
    deltas.extend(reason.map(|reason| done(stop_reason(reason))));
    deltas
}

/// Traduit une raison d'arrêt du service ; toute autre que la limite de longueur termine le tour.
fn stop_reason(reason: &str) -> StopReason {
    match reason {
        "max_tokens" => StopReason::MaxTokens,
        _ => StopReason::EndTurn,
    }
}

/// Construit le delta de fin avec la raison donnée.
fn done(reason: StopReason) -> ChatDelta {
    ChatDelta::Done { reason }
}
