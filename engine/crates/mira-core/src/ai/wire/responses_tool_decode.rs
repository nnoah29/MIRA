//! Format Responses : appels de fonction en flux, réduits à des fragments rangés par `output_index`.

use mira_vocabulary::ChatDelta;
use serde_json::Value;

/// Fragment d'un élément ajouté : seul un `function_call` en porte, avec son identifiant et son nom.
pub(super) fn added_fragment(chunk: &Value) -> Option<ChatDelta> {
    let item = chunk.get("item")?;
    if item.get("type")?.as_str()? != "function_call" {
        return None;
    }
    Some(ChatDelta::ToolFragment {
        index: output_index(chunk)?,
        id: text(item, "call_id"),
        name: text(item, "name"),
        arguments: text(item, "arguments").unwrap_or_default(),
    })
}

/// Fragment d'arguments d'un appel en cours.
pub(super) fn arguments_fragment(chunk: &Value) -> Option<ChatDelta> {
    Some(ChatDelta::ToolFragment {
        index: output_index(chunk)?,
        id: None,
        name: None,
        arguments: chunk.get("delta")?.as_str()?.to_string(),
    })
}

/// Rang de l'élément de sortie.
fn output_index(chunk: &Value) -> Option<u32> {
    u32::try_from(chunk.get("output_index")?.as_u64()?).ok()
}

/// Texte d'un champ de l'élément.
fn text(item: &Value, name: &str) -> Option<String> {
    item.get(name)?.as_str().map(str::to_string)
}
