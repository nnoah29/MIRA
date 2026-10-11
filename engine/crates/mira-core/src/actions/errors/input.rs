//! Erreurs de paramètre et de boîte manquante côté serveur.

use mira_vocabulary::error::codes::{invalid_input, server};
use mira_vocabulary::{ErrorCategory, MiraError};

/// Valeur de paramètre invalide
pub(crate) fn invalid_parameter(message: &str) -> MiraError {
    MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::PARAMETER,
        message,
    )
}

/// Paramètre obligatoire absent
pub(crate) fn missing_parameter(message: &str) -> MiraError {
    MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::MISSING_PARAMETER,
        message,
    )
}

/// Aucune boîte du rôle voulu n'est connue pour ce compte
pub(crate) fn role_mailbox_missing(role: &str) -> MiraError {
    MiraError::new(
        ErrorCategory::Server,
        server::MAILBOX_MISSING,
        format!("aucune boîte « {role} » n'est connue pour ce compte"),
    )
}
