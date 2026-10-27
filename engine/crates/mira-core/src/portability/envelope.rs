//! Enveloppe `mira_export` : type de fichier, version du format, bornes de taille.

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{ErrorCategory, MiraError};
use serde_json::{Map, Value, json};

use super::syntax::{parse, render};

const FORMAT_VERSION: u64 = 1;
const MAX_BYTES: usize = 2_097_152;
const MAX_ITEMS: usize = 500;

/// Erreur d'entrée invalide portant le code donné
fn refusal(code: mira_vocabulary::ErrorCode, message: &str) -> MiraError {
    MiraError::new(ErrorCategory::InvalidInput, code, message)
}

/// Produit le document YAML : en-tête `mira_export` puis les éléments sous leur clé
pub fn wrap(kind: &str, items_key: &str, items: Vec<Value>) -> Result<String, MiraError> {
    let mut root = Map::new();
    root.insert(
        "mira_export".to_string(),
        json!({ "kind": kind, "version": FORMAT_VERSION }),
    );
    root.insert(items_key.to_string(), Value::Array(items));
    render(&Value::Object(root))
}

/// Contrôle la taille, la syntaxe, le type et la version, puis rend les éléments du fichier
pub fn unwrap(text: &str, kind: &str, items_key: &str) -> Result<Vec<Value>, MiraError> {
    if text.len() > MAX_BYTES {
        return Err(refusal(
            invalid_input::EXPORT_TOO_LARGE,
            "fichier d'export trop gros",
        ));
    }
    let root = parse(text.strip_prefix('\u{feff}').unwrap_or(text))?;
    check_header(&root, kind)?;
    match root.get(items_key) {
        Some(Value::Array(items)) if items.len() <= MAX_ITEMS => Ok(items.clone()),
        Some(Value::Array(_)) => Err(refusal(invalid_input::EXPORT_TOO_LARGE, "trop d'éléments")),
        _ => Err(refusal(
            invalid_input::EXPORT_KIND_MISMATCH,
            "liste d'éléments absente",
        )),
    }
}

/// Vérifie l'en-tête : le type attendu d'abord, puis la version du format
fn check_header(root: &Value, kind: &str) -> Result<(), MiraError> {
    let header = root.get("mira_export");
    if header.and_then(|h| h.get("kind")).and_then(Value::as_str) != Some(kind) {
        return Err(refusal(
            invalid_input::EXPORT_KIND_MISMATCH,
            "ce fichier n'est pas de ce type",
        ));
    }
    if header
        .and_then(|h| h.get("version"))
        .and_then(Value::as_u64)
        != Some(FORMAT_VERSION)
    {
        return Err(refusal(
            invalid_input::EXPORT_VERSION_UNSUPPORTED,
            "version du format non prise en charge",
        ));
    }
    Ok(())
}
