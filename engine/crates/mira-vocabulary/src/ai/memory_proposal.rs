//! Proposition de l'assistant sur la mémoire : retenir ou oublier ; rien n'est écrit avant le clic de l'utilisateur.

use serde::{Deserialize, Serialize};

use super::memory::MemoryKind;

/// Ce que l'assistant propose de faire d'un souvenir.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryProposalAction {
    Remember,
    Forget,
}

/// Une proposition : le texte à retenir, ou le souvenir à oublier (`id` et son texte).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryProposal {
    pub action: MemoryProposalAction,
    pub text: String,
    pub kind: MemoryKind,
    pub id: Option<String>,
}
