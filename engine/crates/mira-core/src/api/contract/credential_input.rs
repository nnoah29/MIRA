//! Secret d'un compte transmis à l'ajout : une seule fois, jamais renvoyé (S1–S3, P9).

use mira_vocabulary::AuthMethodKind;
use serde::{Deserialize, Serialize};

use super::secret_input::SecretInput;

/// Identifiant fourni pour ajouter ou tester un compte ; les valeurs secrètes sont masquées par `Debug`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum CredentialInput {
    Password(SecretInput),
    AppPassword(SecretInput),
    #[serde(rename = "oauth_flow")]
    OAuthFlow {
        flow: String,
    },
}

impl CredentialInput {
    /// Méthode d'authentification que ce secret permet d'utiliser
    pub fn auth_kind(&self) -> AuthMethodKind {
        match self {
            Self::Password(_) => AuthMethodKind::Password,
            Self::AppPassword(_) => AuthMethodKind::AppPassword,
            Self::OAuthFlow { .. } => AuthMethodKind::OAuth2,
        }
    }
}
