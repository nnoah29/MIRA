//! Contrôle du libellé d'une connexion d'IA.

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{ErrorCategory, MiraError};

/// Retire les blancs de bord ; un libellé vide est refusé
pub(super) fn validate_label(label: &str) -> Result<String, MiraError> {
    let trimmed = label.trim();
    if !trimmed.is_empty() {
        return Ok(trimmed.to_string());
    }
    Err(MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::PARAMETER,
        "le libellé de la connexion d'IA est vide",
    ))
}
