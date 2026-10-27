#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Documents d'export de test et lecture du code d'une erreur.

use mira_vocabulary::MiraError;
use serde_json::Value;

/// Code de l'erreur, ou échec du test si l'appel a réussi
pub fn code_of(result: Result<Vec<Value>, MiraError>) -> String {
    result.unwrap_err().code
}

/// Document minimal portant le type, la version et les éléments donnés
pub fn document(kind: &str, version: u32, items: &str) -> String {
    format!("mira_export:\n  kind: {kind}\n  version: {version}\nrules: {items}\n")
}
