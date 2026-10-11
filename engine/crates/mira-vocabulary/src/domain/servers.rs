//! Serveurs IMAP et SMTP d'un compte.

use serde::{Deserialize, Serialize};

/// Mode de chiffrement de la connexion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Security {
    Tls,
    StartTls,
    Plaintext,
}

/// Adresse d'un serveur : hôte, port et chiffrement.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServerEndpoint {
    pub host: String,
    pub port: u16,
    pub security: Security,
}

/// Les deux serveurs d'un compte.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServerSettings {
    pub imap: ServerEndpoint,
    pub smtp: ServerEndpoint,
}
