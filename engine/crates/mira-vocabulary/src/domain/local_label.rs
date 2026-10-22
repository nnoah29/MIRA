//! Libellé local : étiquette propre à MIRA, jamais envoyée au serveur, posée sur des conversations.

use serde::{Deserialize, Serialize};

use crate::ids::kinds::{LocalLabelId, ThreadId};

/// Libellé local : nom unique (sans tenir compte de la casse) et couleur ARGB.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalLabel {
    pub id: LocalLabelId,
    pub name: String,
    pub color: u32,
}

/// Libellés locaux posés sur une conversation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalLabelAssignment {
    pub thread: ThreadId,
    pub labels: Vec<LocalLabelId>,
}
