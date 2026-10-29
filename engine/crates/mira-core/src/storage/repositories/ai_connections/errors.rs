//! Erreurs du dépôt des connexions d'IA.

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{ErrorCategory, MiraError};

/// Erreur « connexion d'IA inconnue »
pub(super) fn connection_not_found() -> MiraError {
    MiraError::new(
        ErrorCategory::NotFound,
        not_found::AI_CONNECTION,
        "connexion d'IA inconnue",
    )
}

/// Échoue avec `not_found.ai_connection` si l'écriture n'a touché aucune ligne
pub(super) fn ensure_changed(changed: usize) -> Result<(), MiraError> {
    match changed {
        0 => Err(connection_not_found()),
        _ => Ok(()),
    }
}
