//! Flux SSE au format Anthropic Messages : message_start, deltas, message_delta, message_stop.

use serde_json::{Value, json};

use super::behavior::Behavior;

/// Construit un flux Anthropic ; `usage` vaut `(jetons d'entrée, jetons de sortie)`
pub fn anthropic_stream(texts: &[&str], usage: Option<(u32, u32)>) -> Behavior {
    let (input, output) = usage.unzip();
    let mut chunks = vec![event("message_start", &start(input))];
    for text in texts {
        chunks.push(event("content_block_delta", &text_delta(text)));
    }
    chunks.push(event("message_delta", &message_delta(output)));
    chunks.push(event("message_stop", &json!({"type": "message_stop"})));
    Behavior::Stream {
        chunks,
        chunk_delay_ms: 0,
    }
}

/// Événement SSE : ligne `event:` puis ligne `data:`
pub(super) fn event(name: &str, value: &Value) -> Vec<u8> {
    format!("event: {name}\ndata: {value}\n\n").into_bytes()
}

/// Début de message, avec les jetons d'entrée s'ils sont connus
pub(super) fn start(input: Option<u32>) -> Value {
    let mut message = json!({"id": "msg_test", "type": "message", "role": "assistant"});
    if let Some(tokens) = input {
        message["usage"] = json!({"input_tokens": tokens});
    }
    json!({"type": "message_start", "message": message})
}

/// Fragment de texte d'un bloc de contenu
pub(super) fn text_delta(text: &str) -> Value {
    json!({"type": "content_block_delta", "index": 0, "delta": {"type": "text_delta", "text": text}})
}

/// Raison d'arrêt, avec les jetons de sortie s'ils sont connus
fn message_delta(output: Option<u32>) -> Value {
    let mut value = json!({"type": "message_delta", "delta": {"stop_reason": "end_turn"}});
    if let Some(tokens) = output {
        value["usage"] = json!({"output_tokens": tokens});
    }
    value
}
