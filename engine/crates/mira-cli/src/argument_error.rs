//! Erreur de saisie détectée par le CLI avant tout appel au moteur.

use mira_assembly::mira_vocabulary::error::codes::invalid_input;
use mira_assembly::mira_vocabulary::{ErrorCategory, MiraError};

/// Erreur « entrée invalide » dont le message ne contient jamais de secret.
pub fn invalid_argument(message: &str) -> MiraError {
    MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::PARAMETER,
        message,
    )
}
