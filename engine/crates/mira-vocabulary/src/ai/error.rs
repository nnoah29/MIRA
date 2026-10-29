//! Erreurs d'une discussion avec un service d'IA, converties en erreur du contrat.

use crate::error::category::ErrorCategory;
use crate::error::code::ErrorCode;
use crate::error::codes::{
    authentication, cancelled, incompatible_state, invalid_input, not_found, server,
};
use crate::error::mira_error::MiraError;

/// Échec d'un échange avec le service ; aucune variante ne porte de contenu libre.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AiError {
    #[error("clé refusée par le service d'IA")]
    KeyRejected,
    #[error("modèle d'IA inconnu du service")]
    ModelNotFound,
    #[error("le service d'IA limite le débit des requêtes")]
    RateLimited { retry_after_secs: Option<u64> },
    #[error("texte trop long pour le service d'IA")]
    InputTooLarge,
    #[error("service d'IA indisponible")]
    Unavailable,
    #[error("réponse inattendue du service d'IA")]
    BadResponse,
    #[error("consentement requis avant d'envoyer du texte au service d'IA")]
    ConsentRequired,
    #[error("discussion annulée")]
    Cancelled,
}

impl AiError {
    /// Donne le code stable de l'erreur dans le catalogue.
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::KeyRejected => authentication::AI_KEY_REJECTED,
            Self::ModelNotFound => not_found::AI_MODEL,
            Self::RateLimited { .. } => server::AI_RATE_LIMITED,
            Self::InputTooLarge => invalid_input::AI_INPUT_TOO_LARGE,
            Self::Unavailable => server::AI_UNAVAILABLE,
            Self::BadResponse => server::AI_BAD_RESPONSE,
            Self::ConsentRequired => incompatible_state::AI_CONSENT_REQUIRED,
            Self::Cancelled => cancelled::BY_USER,
        }
    }

    /// Donne la catégorie de l'erreur dans le contrat.
    fn category(&self) -> ErrorCategory {
        match self {
            Self::KeyRejected => ErrorCategory::Authentication,
            Self::ModelNotFound => ErrorCategory::NotFound,
            Self::RateLimited { .. } | Self::Unavailable | Self::BadResponse => {
                ErrorCategory::Server
            }
            Self::InputTooLarge => ErrorCategory::InvalidInput,
            Self::ConsentRequired => ErrorCategory::IncompatibleState,
            Self::Cancelled => ErrorCategory::Cancelled,
        }
    }

    /// Dit si rejouer plus tard peut réussir.
    fn is_retryable(&self) -> bool {
        matches!(self, Self::RateLimited { .. } | Self::Unavailable)
    }
}

impl From<AiError> for MiraError {
    /// Convertit l'erreur d'IA en erreur du contrat, avec un message sans contenu libre.
    fn from(error: AiError) -> Self {
        let mira = MiraError::new(error.category(), error.code(), error.to_string());
        mira.retryable(error.is_retryable())
    }
}
