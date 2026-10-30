//! Décodage de la liste des modèles au format Messages (`{"data":[{"id":..,"created_at":..}]}`).

use mira_vocabulary::{AiError, AiModel};
use serde_json::Value;

use super::rfc3339::unix_seconds;

/// Décode le corps de la liste ; sans champ `data` ou illisible, la réponse est inattendue.
pub fn decode_models(body: &[u8]) -> Result<Vec<AiModel>, AiError> {
    let root: Value = serde_json::from_slice(body).map_err(|_| AiError::BadResponse)?;
    let entries = root
        .get("data")
        .and_then(Value::as_array)
        .ok_or(AiError::BadResponse)?;
    Ok(entries.iter().filter_map(model_entry).collect())
}

/// Lit un modèle ; une entrée sans identifiant texte est écartée, une date illisible devient absente.
fn model_entry(entry: &Value) -> Option<AiModel> {
    let id = entry.get("id")?.as_str()?;
    let created_at = entry.get("created_at").and_then(Value::as_str);
    Some(AiModel {
        id: id.to_string(),
        created_at: created_at.and_then(unix_seconds),
    })
}
