//! Types d'authentification, d'état de l'hôte et d'opérations d'hôte échangés dans les trames.

use serde::{Deserialize, Serialize};

/// Preuve d'identité donnée dans `hello` ; typée pour accueillir d'autres moyens (clé d'appareil en v4).
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Auth {
    LocalToken { token: String },
}

impl std::fmt::Debug for Auth {
    /// N'affiche jamais le jeton
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Auth(..)")
    }
}

/// État de l'hôte annoncé à l'attache.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostState {
    Ready,
    Starting,
    KeyringLocked,
    UpdateRequired,
}

/// Opération réservée à l'hôte, hors du contrat d'API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostOp {
    Shutdown,
    RetryUnlock,
    TestNotification,
}
