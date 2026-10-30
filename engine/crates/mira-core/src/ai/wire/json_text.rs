//! Lecture d'un texte JSON attendu sous forme d'objet : schéma d'outil ou arguments d'un appel.

use serde_json::Value;

/// Rend l'objet JSON que le texte contient, sinon la valeur de repli.
pub(super) fn object_or(text: &str, fallback: Value) -> Value {
    match serde_json::from_str::<Value>(text) {
        Ok(value) if value.is_object() => value,
        _ => fallback,
    }
}
