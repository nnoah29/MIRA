//! Session au trousseau : identifiants du client, jetons et expiration, rangés ensemble en JSON.

use std::fmt;

use mira_vocabulary::error::codes::incompatible_state;
use mira_vocabulary::{ErrorCategory, MiraError};
use serde::{Deserialize, Serialize};

use crate::ports::SecretValue;

/// Jetons d'une connexion ; `Debug` n'affiche ni jeton ni adresse de courriel.
#[derive(Clone, Serialize, Deserialize)]
pub struct AiSession {
    pub client_id: String,
    pub host_id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    #[serde(default)]
    pub email: Option<String>,
}

impl fmt::Debug for AiSession {
    /// Affiche l'échéance seulement
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AiSession")
            .field("expires_at", &self.expires_at)
            .finish_non_exhaustive()
    }
}

impl AiSession {
    /// Sérialise la session en un secret unique pour le trousseau
    pub fn to_secret(&self) -> SecretValue {
        SecretValue::new(serde_json::to_string(self).unwrap_or_default())
    }

    /// Relit une session rangée ; un secret qui n'en est pas une est refusé sans être recopié
    pub fn from_secret(secret: &SecretValue) -> Result<Self, MiraError> {
        serde_json::from_str(secret.expose()).map_err(|_| {
            MiraError::new(
                ErrorCategory::IncompatibleState,
                incompatible_state::AI_KEY_MISSING,
                "session d'IA illisible, reconnexion nécessaire",
            )
        })
    }
}
