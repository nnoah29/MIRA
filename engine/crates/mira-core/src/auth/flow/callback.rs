//! Lecture des paramètres de la redirection : état, code d'autorisation ou refus.

use mira_vocabulary::error::codes::authentication;
use mira_vocabulary::{ErrorCategory, MiraError};

use super::state_check::state_matches;
use crate::auth::form::parse_query;
use crate::ports::SecretValue;

/// Code d'autorisation reçu ; usage unique, jamais affiché par `Debug`.
#[derive(Debug, Clone)]
pub struct AuthorizationCode(SecretValue);

impl AuthorizationCode {
    /// Enveloppe un code reçu du serveur d'autorisation
    pub fn new(code: impl Into<String>) -> Self {
        Self(SecretValue::new(code))
    }

    /// Donne le code, pour l'échange contre des jetons seulement
    pub fn expose(&self) -> &str {
        self.0.expose()
    }
}

/// Analyse la requête de redirection : vérifie `state`, puis renvoie le code ou l'erreur
pub(super) fn parse_callback(
    query: &str,
    expected_state: &str,
) -> Result<AuthorizationCode, MiraError> {
    let pairs = parse_query(query);
    let find = |name: &str| {
        pairs
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    };
    if !state_matches(find("state"), expected_state) {
        return Err(failure(
            authentication::OAUTH_STATE_MISMATCH,
            "paramètre state inattendu",
        ));
    }
    match (find("error"), find("code")) {
        (Some("access_denied"), _) => Err(failure(
            authentication::OAUTH_CANCELLED,
            "autorisation refusée par l'utilisateur",
        )),
        (Some(_), _) | (None, None) => Err(failure(
            authentication::REJECTED,
            "autorisation refusée par le service",
        )),
        (None, Some(code)) => Ok(AuthorizationCode::new(code)),
    }
}

/// Erreur d'authentification sans valeur issue de la requête
fn failure(code: mira_vocabulary::ErrorCode, message: &str) -> MiraError {
    MiraError::new(ErrorCategory::Authentication, code, message)
}
