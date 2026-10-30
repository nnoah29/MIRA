//! Emplacement d'un secret dans le trousseau : compte de messagerie ou connexion d'IA.

use serde::{Deserialize, Serialize};

use super::auth::SecretRef;
use crate::ids::kinds::{AiConnectionId, ProfileId};

/// Désigne l'endroit du trousseau où vit un secret, sans jamais le contenir.
pub trait SecretLocation: Send + Sync {
    /// Donne la clé de stockage unique de cet emplacement
    fn storage_key(&self) -> String;
}

/// Référence à la clé d'API d'une connexion d'IA, qui n'appartient à aucun compte de messagerie.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AiSecretRef {
    pub profile: ProfileId,
    pub connection: AiConnectionId,
}

impl SecretLocation for SecretRef {
    /// Donne la clé `mira/<profil>/<compte>/<nature>`
    fn storage_key(&self) -> String {
        SecretRef::storage_key(self)
    }
}

impl SecretLocation for AiSecretRef {
    /// Donne la clé `mira/<profil>/ai/<connexion>/api_key`
    fn storage_key(&self) -> String {
        format!("mira/{}/ai/{}/api_key", self.profile, self.connection)
    }
}
