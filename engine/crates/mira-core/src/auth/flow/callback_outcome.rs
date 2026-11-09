//! Issue d'une redirection d'autorisation : code reçu, refus de l'utilisateur ou échec du service.

use mira_vocabulary::error::codes::authentication;
use mira_vocabulary::{ErrorCategory, MiraError};

use super::callback::AuthorizationCode;

/// Issue de la redirection : refus, échec du service, ou code (avec le client émis s'il y en a un)
pub(super) fn outcome(
    error: Option<&str>,
    code: Option<&str>,
    issued_client: Option<&str>,
) -> Result<AuthorizationCode, MiraError> {
    match (error, code) {
        (Some("access_denied"), _) => Err(failure(
            authentication::OAUTH_CANCELLED,
            "autorisation refusée par l'utilisateur",
        )),
        (Some(_), _) | (None, None) => Err(failure(
            authentication::REJECTED,
            "autorisation refusée par le service",
        )),
        (None, Some(code)) => Ok(AuthorizationCode::new(code).with_issued_client(issued_client)),
    }
}

/// Erreur d'authentification sans valeur issue de la requête
pub(super) fn failure(code: mira_vocabulary::ErrorCode, message: &str) -> MiraError {
    MiraError::new(ErrorCategory::Authentication, code, message)
}
