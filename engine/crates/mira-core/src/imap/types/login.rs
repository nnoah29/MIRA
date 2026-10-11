//! Identifiants de connexion et réglages du client IMAP.

use std::time::Duration;

use crate::ports::SecretValue;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Secret présenté au serveur : mot de passe ou jeton d'accès OAuth2 (jamais affiché).
#[derive(Debug, Clone)]
pub enum ImapCredential {
    Password(SecretValue),
    XOAuth2(SecretValue),
}

/// Utilisateur et secret d'une connexion IMAP.
#[derive(Debug, Clone)]
pub struct ImapLogin {
    pub user: String,
    pub credential: ImapCredential,
}

/// Réglages du client : délai d'attente de chaque échange avec le serveur.
#[derive(Debug, Clone)]
pub struct ImapOptions {
    pub timeout: Duration,
}

impl Default for ImapOptions {
    /// Délai de 30 secondes par défaut.
    fn default() -> Self {
        Self {
            timeout: DEFAULT_TIMEOUT,
        }
    }
}
