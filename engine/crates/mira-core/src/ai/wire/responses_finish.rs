//! Format Responses : fin de réponse (consommation et raison d'arrêt) lue dans l'événement de clôture.

use mira_vocabulary::{ChatDelta, StopReason};
use serde_json::Value;

/// Consommation (si portée) puis arrêt.
pub(super) fn finish(chunk: &Value, reason: StopReason) -> Vec<ChatDelta> {
    let mut deltas: Vec<ChatDelta> = usage_delta(chunk).into_iter().collect();
    deltas.push(ChatDelta::Done { reason });
    deltas
}

/// Consommation de la réponse, un champ absent valant 0.
fn usage_delta(chunk: &Value) -> Option<ChatDelta> {
    let usage = chunk.pointer("/response/usage")?.as_object()?;
    let count = |name: &str| {
        let value = usage.get(name).and_then(Value::as_u64).unwrap_or(0);
        u32::try_from(value).unwrap_or(u32::MAX)
    };
    Some(ChatDelta::Usage {
        input_tokens: count("input_tokens"),
        output_tokens: count("output_tokens"),
    })
}

/// Fin normale, ou demande d'outil quand la sortie contient un appel de fonction.
pub(super) fn completed_reason(chunk: &Value) -> StopReason {
    let output = chunk.pointer("/response/output").and_then(Value::as_array);
    let has_call = output.is_some_and(|items| {
        items
            .iter()
            .any(|item| item.get("type").and_then(Value::as_str) == Some("function_call"))
    });
    if has_call {
        StopReason::ToolUse
    } else {
        StopReason::EndTurn
    }
}

/// Réponse tronquée : borne de sortie atteinte, sinon fin normale.
pub(super) fn incomplete_reason(chunk: &Value) -> StopReason {
    let reason = chunk.pointer("/response/incomplete_details/reason");
    match reason.and_then(Value::as_str) {
        Some("max_output_tokens") => StopReason::MaxTokens,
        _ => StopReason::EndTurn,
    }
}
