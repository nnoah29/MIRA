//! Décodage d'un événement SSE au format OpenAI Chat en deltas, sans copier de contenu d'erreur.

use mira_vocabulary::{AiError, ChatDelta, StopReason};
use serde_json::Value;

use super::openai_tool_decode::fragments;
use crate::ai::sse::SseEvent;

const DONE_MARKER: &str = "[DONE]";

/// Décode un événement ; texte, appels d'outils, consommation puis arrêt, dans cet ordre.
pub fn decode_chat_event(event: &SseEvent) -> Result<Vec<ChatDelta>, AiError> {
    let data = event.data.trim();
    if data.is_empty() || data == DONE_MARKER {
        return Ok(Vec::new());
    }
    let chunk: Value = serde_json::from_str(data).map_err(|_| AiError::BadResponse)?;
    let mut deltas = Vec::new();
    deltas.extend(text_delta(&chunk));
    deltas.extend(fragments(&chunk));
    deltas.extend(usage_delta(&chunk));
    deltas.extend(finish_delta(&chunk));
    Ok(deltas)
}

/// Extrait le texte non vide du premier choix.
fn text_delta(chunk: &Value) -> Option<ChatDelta> {
    let text = chunk.pointer("/choices/0/delta/content")?.as_str()?;
    (!text.is_empty()).then(|| ChatDelta::Text {
        text: text.to_string(),
    })
}

/// Extrait la consommation portée par le morceau, un champ absent valant 0.
fn usage_delta(chunk: &Value) -> Option<ChatDelta> {
    let usage = chunk.get("usage")?.as_object()?;
    let count = |name: &str| token_count(usage.get(name));
    Some(ChatDelta::Usage {
        input_tokens: count("prompt_tokens"),
        output_tokens: count("completion_tokens"),
    })
}

/// Convertit un compteur JSON en u32, 0 s'il est absent ou invalide, saturé s'il est trop grand.
fn token_count(value: Option<&Value>) -> u32 {
    let count = value.and_then(Value::as_u64).unwrap_or(0);
    u32::try_from(count).unwrap_or(u32::MAX)
}

/// Traduit la raison d'arrêt du premier choix, si le morceau en porte une.
fn finish_delta(chunk: &Value) -> Option<ChatDelta> {
    let reason = chunk.pointer("/choices/0/finish_reason")?.as_str()?;
    let reason = match reason {
        "length" => StopReason::MaxTokens,
        "tool_calls" => StopReason::ToolUse,
        _ => StopReason::EndTurn,
    };
    Some(ChatDelta::Done { reason })
}
