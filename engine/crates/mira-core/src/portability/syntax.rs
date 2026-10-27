//! Lecture et écriture YAML : les erreurs donnent la position, jamais un extrait du contenu.

use mira_vocabulary::error::codes::{internal, invalid_input};
use mira_vocabulary::{ErrorCategory, MiraError};
use serde_json::Value;

/// Analyse un texte YAML en valeur JSON ; l'erreur porte ligne et colonne quand elles sont connues
pub(super) fn parse(text: &str) -> Result<Value, MiraError> {
    yaml_serde::from_str::<Value>(text).map_err(|error| {
        let message = match error.location() {
            Some(at) => format!(
                "YAML invalide à la ligne {}, colonne {}",
                at.line(),
                at.column()
            ),
            None => "YAML invalide ou trop répétitif".to_string(),
        };
        MiraError::new(
            ErrorCategory::InvalidInput,
            invalid_input::YAML_SYNTAX,
            message,
        )
    })
}

/// Écrit une valeur JSON en texte YAML
pub(super) fn render(value: &Value) -> Result<String, MiraError> {
    yaml_serde::to_string(value).map_err(|_| {
        MiraError::new(
            ErrorCategory::Internal,
            internal::BUG,
            "export non sérialisable en YAML",
        )
    })
}
