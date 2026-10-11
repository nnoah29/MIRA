//! Conversion des échecs du point d'accès de jetons en erreurs du contrat, sans secret.

use mira_vocabulary::error::codes::{authentication, network, server};
use mira_vocabulary::{ErrorCategory, MiraError, SuggestedAction};
use serde::Deserialize;

#[derive(Deserialize)]
struct RawFailure {
    error: Option<String>,
}

/// Convertit une réponse HTTP d'échec : `invalid_grant` signifie jeton ou code révoqué
pub(super) fn map_token_failure(status: u16, body: &[u8]) -> MiraError {
    let failure: Option<RawFailure> = serde_json::from_slice(body).ok();
    let reason = failure.and_then(|raw| raw.error);
    if reason.as_deref() == Some("invalid_grant") {
        return MiraError::new(
            ErrorCategory::Authentication,
            authentication::TOKEN_REVOKED,
            "jeton ou code d'autorisation révoqué",
        )
        .with_action(SuggestedAction::Reauthenticate);
    }
    if status >= 500 {
        let message = format!("service d'autorisation indisponible ({status})");
        return MiraError::new(ErrorCategory::Server, server::REJECTED, message).retryable(true);
    }
    MiraError::new(
        ErrorCategory::Authentication,
        authentication::REJECTED,
        "demande de jetons refusée",
    )
}

/// Convertit une erreur de transport HTTP, sans reprendre son texte
pub(super) fn map_http_error(error: &reqwest::Error) -> MiraError {
    let (code, message) = if error.is_timeout() {
        (
            network::TIMEOUT,
            "délai dépassé avec le service d'autorisation",
        )
    } else if error.is_connect() {
        (network::UNREACHABLE, "service d'autorisation injoignable")
    } else {
        (
            network::CONNECTION_LOST,
            "échange interrompu avec le service d'autorisation",
        )
    };
    MiraError::new(ErrorCategory::Network, code, message).retryable(true)
}
