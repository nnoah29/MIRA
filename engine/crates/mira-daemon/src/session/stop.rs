//! Pourquoi un flux s'arrête avant sa fin naturelle.

use mira_protocol::EndReason;

/// Raison d'un arrêt demandé : annulation du client ou de la connexion, ou client trop lent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stop {
    Cancelled,
    Lagged,
}

impl From<Stop> for EndReason {
    /// La raison de fin annoncée au client
    fn from(stop: Stop) -> Self {
        match stop {
            Stop::Cancelled => EndReason::Cancelled,
            Stop::Lagged => EndReason::Lagged,
        }
    }
}
