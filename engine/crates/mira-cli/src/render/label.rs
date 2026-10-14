//! Libellé court d'une valeur sérialisable (état, type d'événement, méthode d'authentification).

use serde::Serialize;
use serde_json::Value;

/// Donne le texte d'une chaîne JSON, le champ `type` d'un objet, ou le JSON compact sinon.
pub fn json_label<T: Serialize>(value: &T) -> String {
    match serde_json::to_value(value) {
        Ok(Value::String(text)) => text,
        Ok(Value::Object(map)) => match map.get("type").and_then(Value::as_str) {
            Some(tag) => tag.to_string(),
            None => Value::Object(map).to_string(),
        },
        Ok(other) => other.to_string(),
        Err(_) => "?".to_string(),
    }
}
