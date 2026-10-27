//! Lecture et écriture YAML : les erreurs donnent la position, jamais un extrait du contenu.

use std::sync::LazyLock;

use mira_vocabulary::error::codes::{internal, invalid_input};
use mira_vocabulary::{ErrorCategory, MiraError};
use regex::Regex;
use serde_json::Value;

static ANCHOR: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"(?m)(^|[\s\[\{,:\-])&\S").ok());

/// Vrai si le texte contient peut-être une ancre YAML (`&nom`), donc des alias qui multiplient le document
fn may_hold_anchor(text: &str) -> bool {
    ANCHOR.as_ref().is_none_or(|anchor| anchor.is_match(text))
}

/// Analyse un texte YAML en valeur JSON ; ancres refusées, l'erreur porte ligne et colonne quand elles sont connues
pub(super) fn parse(text: &str) -> Result<Value, MiraError> {
    if may_hold_anchor(text) {
        return Err(MiraError::new(
            ErrorCategory::InvalidInput,
            invalid_input::YAML_SYNTAX,
            "les ancres et alias YAML (&, *) ne sont pas pris en charge",
        ));
    }
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
