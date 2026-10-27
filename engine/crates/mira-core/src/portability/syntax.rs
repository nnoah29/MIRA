//! Lecture et écriture YAML : les erreurs donnent la position, jamais un extrait du contenu.

use mira_vocabulary::error::codes::{internal, invalid_input};
use mira_vocabulary::{ErrorCategory, MiraError};
use serde_json::Value;

use super::scan::{Problem, scan};

/// Refus d'un texte YAML : la cause en mots, sans aucun extrait du contenu, avec la position
fn refusal(problem: Problem, line: usize, column: usize) -> MiraError {
    let what = match problem {
        Problem::Alias => "alias YAML (*) non pris en charge",
        Problem::DuplicateKey => "clé en double",
        Problem::Syntax => "YAML invalide",
    };
    let message = format!("{what} à la ligne {line}, colonne {column}");
    MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::YAML_SYNTAX,
        message,
    )
}

/// Analyse un texte YAML en valeur JSON ; alias et clés en double refusés, l'erreur porte ligne et colonne
pub(super) fn parse(text: &str) -> Result<Value, MiraError> {
    scan(text).map_err(|(problem, line, column)| refusal(problem, line, column))?;
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
