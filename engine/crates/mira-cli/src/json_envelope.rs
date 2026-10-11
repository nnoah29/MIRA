//! Enveloppe JSON versionnée de la sortie `--json` (12.9, invariant 33).

use mira_assembly::mira_vocabulary::error::codes::internal;
use mira_assembly::mira_vocabulary::{CONTRACT_VERSION, ErrorCategory, MiraError};
use serde_json::{Map, Value};

/// Convertit une valeur sérialisable en JSON, ou en erreur interne si elle ne l'est pas.
pub fn to_json_value<T: serde::Serialize>(value: &T) -> Result<Value, MiraError> {
    serde_json::to_value(value).map_err(|_| {
        MiraError::new(
            ErrorCategory::Internal,
            internal::BUG,
            "sérialisation JSON impossible",
        )
    })
}

/// Assemble un objet JSON à partir de couples (clé, valeur).
pub fn object(entries: Vec<(&str, Value)>) -> Value {
    let mut map = Map::new();
    for (key, value) in entries {
        map.insert(key.to_string(), value);
    }
    Value::Object(map)
}

/// Enveloppe de succès : version du contrat, `ok` vrai et les données.
pub fn success_envelope(data: Value) -> Value {
    let version = to_json_value(&CONTRACT_VERSION).unwrap_or(Value::Null);
    object(vec![
        ("contract_version", version),
        ("ok", Value::Bool(true)),
        ("data", data),
    ])
}

/// Enveloppe d'échec : version du contrat, `ok` faux et l'erreur structurée.
pub fn error_envelope(error: &MiraError) -> Value {
    let version = to_json_value(&CONTRACT_VERSION).unwrap_or(Value::Null);
    let payload = to_json_value(error).unwrap_or(Value::Null);
    object(vec![
        ("contract_version", version),
        ("ok", Value::Bool(false)),
        ("error", payload),
    ])
}
