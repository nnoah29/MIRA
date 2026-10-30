//! Tour d'IA au format OpenAI Chat qui demande des outils : arguments reçus en plusieurs morceaux.

use serde_json::{Value, json};

use super::behavior::Behavior;
use super::tool_turns_anthropic::{ToolCallSpec, pieces};

/// Flux d'un tour qui s'arrête pour outils ; `usage` vaut `(jetons d'entrée, jetons de sortie)`.
pub fn openai_tool_turn(
    text: &str,
    calls: &[ToolCallSpec<'_>],
    usage: Option<(u32, u32)>,
) -> Behavior {
    let mut chunks = Vec::new();
    if !text.is_empty() {
        chunks.push(data(&json!({"choices": [{"delta": {"content": text}}]})));
    }
    for (index, (id, name, arguments)) in calls.iter().enumerate() {
        chunks.extend(call_chunks(index as u32, id, name, arguments));
    }
    chunks.push(data(
        &json!({"choices": [{"delta": {}, "finish_reason": "tool_calls"}]}),
    ));
    if let Some((prompt, completion)) = usage {
        let counts = json!({"prompt_tokens": prompt, "completion_tokens": completion});
        chunks.push(data(&json!({"choices": [], "usage": counts})));
    }
    chunks.push(b"data: [DONE]\n\n".to_vec());
    Behavior::Stream {
        chunks,
        chunk_delay_ms: 0,
    }
}

/// Premier morceau (identifiant, nom) puis morceaux d'arguments d'un appel.
fn call_chunks(index: u32, id: &str, name: &str, arguments: &str) -> Vec<Vec<u8>> {
    let function = json!({"name": name, "arguments": ""});
    let first = json!({"index": index, "id": id, "type": "function", "function": function});
    let mut chunks = vec![data(&tool_calls(first))];
    for piece in pieces(arguments) {
        let part = json!({"index": index, "function": {"arguments": piece}});
        chunks.push(data(&tool_calls(part)));
    }
    chunks
}

/// Morceau portant un seul élément de `tool_calls`.
fn tool_calls(call: Value) -> Value {
    json!({"choices": [{"delta": {"tool_calls": [call]}}]})
}

/// Événement SSE à une ligne `data:`.
fn data(value: &Value) -> Vec<u8> {
    format!("data: {value}\n\n").into_bytes()
}
