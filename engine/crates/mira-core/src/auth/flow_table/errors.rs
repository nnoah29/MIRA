//! Erreur d'usage d'un flux OAuth absent de la table.

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{ErrorCategory, MiraError};

/// Erreur d'un flux inconnu, déjà consommé, expiré ou pas encore terminé
pub(super) fn unknown_flow() -> MiraError {
    MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::PARAMETER,
        "flux d'autorisation inconnu, expiré, en cours ou déjà utilisé",
    )
}
