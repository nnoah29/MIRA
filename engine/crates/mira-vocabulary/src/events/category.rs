//! Catégorie d'un événement, utilisée pour s'abonner.

use serde::{Deserialize, Serialize};

/// Famille d'événements ; un abonnement peut en choisir plusieurs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventCategory {
    Engine,
    Account,
    Sync,
    Mailbox,
    Message,
    Thread,
    Action,
    Send,
    Alert,
    Log,
}
