//! Catégorie d'une erreur du contrat (12.6).

use serde::{Deserialize, Serialize};

/// Famille d'erreur ; le client choisit son comportement d'après elle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    InvalidInput,
    NotFound,
    IncompatibleState,
    Network,
    Tls,
    Authentication,
    Server,
    LocalStorage,
    Secrets,
    Cancelled,
    Internal,
}

impl ErrorCategory {
    /// Donne le nom stable de la catégorie, identique à sa forme sérialisée.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InvalidInput => "invalid_input",
            Self::NotFound => "not_found",
            Self::IncompatibleState => "incompatible_state",
            Self::Network => "network",
            Self::Tls => "tls",
            Self::Authentication => "authentication",
            Self::Server => "server",
            Self::LocalStorage => "local_storage",
            Self::Secrets => "secrets",
            Self::Cancelled => "cancelled",
            Self::Internal => "internal",
        }
    }
}
