//! Tour d'IA au format Anthropic Messages qui demande des outils : arguments reçus en plusieurs morceaux.

use serde_json::{Value, json};

use super::anthropic_body::{event, start, text_delta};
use super::behavior::Behavior;

/// Appel joué : identifiant, nom de l'outil et arguments JSON en texte.
pub type ToolCallSpec<'a> = (&'a str, &'a str, &'a str);

/// Flux d'un tour qui s'arrête pour outils ; `usage` vaut `(jetons d'entrée, jetons de sortie)`.
pub fn anthropic_tool_turn(
    text: &str,
    calls: &[ToolCallSpec<'_>],
    usage: Option<(u32, u32)>,
) -> Behavior {
    let (input, output) = usage.unzip();
    let mut chunks = vec![event("message_start", &start(input))];
    if !text.is_empty() {
        chunks.push(event("content_block_delta", &text_delta(text)));
    }
    for (offset, (id, name, arguments)) in calls.iter().enumerate() {
        chunks.extend(call_events(offset as u32 + 1, id, name, arguments));
    }
    chunks.push(event("message_delta", &tool_use_end(output)));
    chunks.push(event("message_stop", &json!({"type": "message_stop"})));
    Behavior::Stream {
        chunks,
        chunk_delay_ms: 0,
    }
}

/// Morceaux d'au plus six caractères, pour que les arguments arrivent coupés.
pub(super) fn pieces(arguments: &str) -> Vec<String> {
    let chars: Vec<char> = arguments.chars().collect();
    chars
        .chunks(6)
        .map(|chunk| chunk.iter().collect())
        .collect()
}

/// Début de bloc, morceaux d'arguments puis fin de bloc d'un appel.
fn call_events(index: u32, id: &str, name: &str, arguments: &str) -> Vec<Vec<u8>> {
    let block = json!({"type": "tool_use", "id": id, "name": name, "input": {}});
    let opening = json!({"type": "content_block_start", "index": index, "content_block": block});
    let mut events = vec![event("content_block_start", &opening)];
    for piece in pieces(arguments) {
        let delta = json!({"type": "input_json_delta", "partial_json": piece});
        let value = json!({"type": "content_block_delta", "index": index, "delta": delta});
        events.push(event("content_block_delta", &value));
    }
    events.push(event(
        "content_block_stop",
        &json!({"type": "content_block_stop", "index": index}),
    ));
    events
}

/// Arrêt pour outils, avec les jetons de sortie s'ils sont connus.
fn tool_use_end(output: Option<u32>) -> Value {
    let mut value = json!({"type": "message_delta", "delta": {"stop_reason": "tool_use"}});
    if let Some(tokens) = output {
        value["usage"] = json!({"output_tokens": tokens});
    }
    value
}
