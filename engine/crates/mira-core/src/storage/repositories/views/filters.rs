//! Filtres de liste et de recherche : tous facultatifs, combinés par « et ».

use mira_vocabulary::{MailboxId, Timestamp};
use serde::{Deserialize, Serialize};

/// Filtres communs aux listes et à la recherche ; `since` et `until` sont inclusifs et portent sur la date de tri.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageFilters {
    pub unread_only: bool,
    pub flagged_only: bool,
    pub has_attachments: bool,
    pub from_contains: Option<String>,
    pub since: Option<Timestamp>,
    pub until: Option<Timestamp>,
}

/// Filtres de la recherche : les filtres communs, le destinataire et une boîte.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchFilters {
    pub common: MessageFilters,
    pub to_contains: Option<String>,
    pub mailbox: Option<MailboxId>,
}

impl SearchFilters {
    /// Aucun filtre n'est posé : la recherche ne se restreint à rien d'autre que le texte.
    pub fn is_empty(&self) -> bool {
        *self == SearchFilters::default()
    }
}
