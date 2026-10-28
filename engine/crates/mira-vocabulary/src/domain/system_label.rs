//! Libellés système : marqueurs de conversation que le moteur pose seul (à répondre, en attente, mentions).

use serde::{Deserialize, Serialize};

use crate::ids::kinds::ThreadId;

/// Sorte de libellé système ; l'ensemble est fixe et connu du moteur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemLabelKind {
    Needs,
    Waiting,
    Mention,
}

impl SystemLabelKind {
    /// Toutes les sortes, dans l'ordre d'affichage.
    pub const ALL: [SystemLabelKind; 3] = [Self::Needs, Self::Waiting, Self::Mention];

    /// Clé stable, utilisée en base et dans le contrat.
    pub fn key(self) -> &'static str {
        match self {
            Self::Needs => "needs",
            Self::Waiting => "waiting",
            Self::Mention => "mention",
        }
    }

    /// Retrouve une sorte par sa clé.
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.key() == key)
    }
}

/// Libellés système portés par une conversation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemLabelAssignment {
    pub thread: ThreadId,
    pub kinds: Vec<SystemLabelKind>,
}
