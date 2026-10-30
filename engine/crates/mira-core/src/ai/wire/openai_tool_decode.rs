//! Format OpenAI Chat : appels d'outils en flux, réduits à des fragments rangés par `index`.

use mira_vocabulary::ChatDelta;
use serde_json::Value;

/// Fragments portés par le premier choix du morceau, dans l'ordre reçu.
pub(super) fn fragments(chunk: &Value) -> Vec<ChatDelta> {
    let calls = chunk
        .pointer("/choices/0/delta/tool_calls")
        .and_then(Value::as_array);
    calls
        .into_iter()
        .flatten()
        .filter_map(fragment_of)
        .collect()
}

/// Fragment d'un élément de `tool_calls` ; sans rang, il est ignoré.
fn fragment_of(call: &Value) -> Option<ChatDelta> {
    let index = u32::try_from(call.get("index")?.as_u64()?).ok()?;
    let text = |pointer: &str| call.pointer(pointer)?.as_str().map(str::to_string);
    Some(ChatDelta::ToolFragment {
        index,
        id: text("/id"),
        name: text("/function/name"),
        arguments: text("/function/arguments").unwrap_or_default(),
    })
}
