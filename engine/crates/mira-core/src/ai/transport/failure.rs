//! Traduction d'un statut HTTP d'échec en `AiError`, sans jamais consulter le corps de la réponse.

use mira_vocabulary::AiError;

use super::retry_after::parse_retry_after;

/// Traduit un statut d'échec et l'en-tête `retry-after` éventuel en erreur d'IA.
pub fn map_status(status: u16, retry_after: Option<&str>) -> AiError {
    match status {
        401 | 403 => AiError::KeyRejected,
        404 => AiError::ModelNotFound,
        429 => AiError::RateLimited {
            retry_after_secs: retry_after.and_then(parse_retry_after),
        },
        400 | 413 | 422 => AiError::InputTooLarge,
        500..=599 => AiError::Unavailable,
        _ => AiError::BadResponse,
    }
}
