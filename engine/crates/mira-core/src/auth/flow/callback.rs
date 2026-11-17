//! Lecture des paramètres de la redirection : état, code d'autorisation ou refus.

use mira_vocabulary::MiraError;
use mira_vocabulary::error::codes::authentication;

use super::callback_outcome::{failure, outcome};
use super::state_check::state_matches;
use crate::auth::form::parse_query;
use crate::ports::SecretValue;

/// Code d'autorisation reçu ; usage unique, jamais affiché par `Debug`.
#[derive(Debug, Clone)]
pub struct AuthorizationCode(SecretValue, Option<String>);

impl AuthorizationCode {
    /// Enveloppe un code reçu du serveur d'autorisation
    pub fn new(code: impl Into<String>) -> Self {
        Self(SecretValue::new(code), None)
    }

    /// Joint l'identifiant de client émis par le service lors d'une inscription
    pub fn with_issued_client(mut self, client_id: Option<&str>) -> Self {
        self.1 = client_id.map(str::to_string);
        self
    }

    /// Identifiant de client émis, quand la redirection en portait un
    pub fn issued_client_id(&self) -> Option<&str> {
        self.1.as_deref()
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
    outcome(find("error"), find("code"), find("client_id"))
}
