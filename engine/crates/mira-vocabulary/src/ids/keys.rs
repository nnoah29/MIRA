//! Clés non opaques : provider, idempotence, curseur de pagination, numéro d'événement.

use serde::{Deserialize, Serialize};

/// Clé du catalogue des providers (`gmail`, `outlook`, `yahoo`, `generic`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ProviderId(pub String);

impl ProviderId {
    /// Construit une clé de provider à partir de son nom.
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }

    /// Donne le nom du provider.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Clé d'idempotence fournie par le client pour une commande (12.2.3).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IdempotencyKey(pub String);

/// Curseur opaque de pagination.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Cursor(pub String);

/// Numéro d'un événement, strictement croissant dans un moteur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EventSeq(pub u64);
