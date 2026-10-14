//! Classement des erreurs du rejeu : passagère, d'authentification ou définitive (10.7).

use mira_vocabulary::error::codes::secrets;
use mira_vocabulary::{ErrorCategory, MiraError};

/// Ce que l'erreur change au sort de l'action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FailureClass {
    Transient,
    Authentication,
    Definitive,
}

/// Classe l'erreur d'une tentative ; seule une erreur définitive annule l'effet local
pub(crate) fn classify(error: &MiraError) -> FailureClass {
    match error.category {
        ErrorCategory::Authentication => FailureClass::Authentication,
        ErrorCategory::Secrets if error.code == secrets::SECRET_MISSING.0 => {
            FailureClass::Authentication
        }
        ErrorCategory::Secrets
        | ErrorCategory::Network
        | ErrorCategory::Tls
        | ErrorCategory::LocalStorage
        | ErrorCategory::Internal
        | ErrorCategory::Cancelled => FailureClass::Transient,
        ErrorCategory::Server if error.retryable => FailureClass::Transient,
        ErrorCategory::Server
        | ErrorCategory::InvalidInput
        | ErrorCategory::NotFound
        | ErrorCategory::IncompatibleState => FailureClass::Definitive,
    }
}
