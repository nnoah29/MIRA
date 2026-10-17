//! Erreurs du module de profil.

use mira_vocabulary::error::codes::local_storage;
use mira_vocabulary::{ErrorCategory, MiraError};

/// Convertit une erreur d'entrée-sortie en erreur du contrat, sans détail système
pub(crate) fn io_error(action: &str, error: &std::io::Error) -> MiraError {
    MiraError::new(
        ErrorCategory::LocalStorage,
        local_storage::IO,
        format!("{action} : {}", error.kind()),
    )
}
