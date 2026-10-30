//! Décodage de la liste des modèles au format OpenAI Chat (`{"data":[{"id":..,"created":..}]}`).

use mira_vocabulary::{AiError, AiModel};
use serde_json::Value;

/// Décode le corps de la liste ; sans champ `data` ou illisible, la réponse est inattendue.
pub fn decode_models(body: &[u8]) -> Result<Vec<AiModel>, AiError> {
    let root: Value = serde_json::from_slice(body).map_err(|_| AiError::BadResponse)?;
    let entries = root
        .get("data")
        .and_then(Value::as_array)
        .ok_or(AiError::BadResponse)?;
    Ok(entries.iter().filter_map(model_entry).collect())
}

/// Lit un modèle ; une entrée sans identifiant texte est écartée.
fn model_entry(entry: &Value) -> Option<AiModel> {
    let id = entry.get("id")?.as_str()?;
    Some(AiModel {
        id: id.to_string(),
        created_at: entry.get("created").and_then(Value::as_i64),
    })
}
