//! Décodage de la liste des modèles au format Responses : modèles listés, identifiés par `slug`.

use mira_vocabulary::{AiError, AiModel};
use serde_json::Value;

/// Décode le corps ; la liste est sous `data` ou `models`, sinon la réponse est inattendue.
pub fn decode_models(body: &[u8]) -> Result<Vec<AiModel>, AiError> {
    let root: Value = serde_json::from_slice(body).map_err(|_| AiError::BadResponse)?;
    let entries = ["data", "models"]
        .iter()
        .find_map(|name| root.get(*name).and_then(Value::as_array))
        .ok_or(AiError::BadResponse)?;
    Ok(entries.iter().filter_map(model_entry).collect())
}

/// Lit un modèle listé ; une entrée masquée ou sans `slug` est écartée.
fn model_entry(entry: &Value) -> Option<AiModel> {
    if entry.get("visibility")?.as_str()? != "list" {
        return None;
    }
    let slug = entry.get("slug")?.as_str()?;
    Some(AiModel {
        id: slug.to_string(),
        created_at: entry.get("created").and_then(Value::as_i64),
    })
}
