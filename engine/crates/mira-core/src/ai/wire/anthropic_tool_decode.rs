//! Format Messages : appels d'outils en flux, réduits à des fragments rangés par bloc.

use mira_vocabulary::ChatDelta;
use serde_json::Value;

/// Fragment d'appel porté par un début de bloc d'outil ou par un morceau d'arguments.
pub(super) fn fragment(payload: &Value) -> Option<ChatDelta> {
    let index = u32::try_from(payload.get("index")?.as_u64()?).ok()?;
    match payload.get("type")?.as_str()? {
        "content_block_start" => start_fragment(payload, index),
        "content_block_delta" => json_fragment(payload, index),
        _ => None,
    }
}

/// Identifiant et nom d'un bloc de type `tool_use`, sans argument encore.
fn start_fragment(payload: &Value, index: u32) -> Option<ChatDelta> {
    let block = payload.get("content_block")?;
    if block.get("type")?.as_str()? != "tool_use" {
        return None;
    }
    Some(ChatDelta::ToolFragment {
        index,
        id: block.get("id")?.as_str().map(str::to_string),
        name: block.get("name")?.as_str().map(str::to_string),
        arguments: String::new(),
    })
}

/// Morceau non vide d'arguments d'un delta de type `input_json_delta`.
fn json_fragment(payload: &Value, index: u32) -> Option<ChatDelta> {
    let delta = payload.get("delta")?;
    if delta.get("type")?.as_str()? != "input_json_delta" {
        return None;
    }
    let partial = delta.get("partial_json")?.as_str()?;
    (!partial.is_empty()).then(|| ChatDelta::ToolFragment {
        index,
        id: None,
        name: None,
        arguments: partial.to_string(),
    })
}
