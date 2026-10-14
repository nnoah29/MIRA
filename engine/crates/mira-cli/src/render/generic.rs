//! Rendu humain provisoire des réponses de lecture et de maintenance : JSON indenté.

use serde::Serialize;

/// Texte JSON indenté d'une réponse ; `?` si elle ne se sérialise pas
pub fn pretty_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "?".to_string())
}
