//! Résultats d'outil propres à la mémoire : proposition retenir ou oublier, et souvenirs rappelés.

use mira_vocabulary::{ContentKind, MemoryProposal, ToolState};

use super::run_result::ToolRun;

impl ToolRun {
    /// Proposition sur la mémoire : la phrase fixe renvoyée au modèle et la proposition pour l'application.
    pub fn memory_proposed(text: String, memory: MemoryProposal) -> Self {
        Self {
            state: ToolState::Ok,
            results: 1,
            memory: Some(memory),
            ..Self::error(text)
        }
    }

    /// Souvenirs rappelés : le texte rendu au modèle, typé « mémoire », avec leur nombre.
    pub fn recalled(text: String, count: u32) -> Self {
        Self {
            kind: ContentKind::Memory,
            state: ToolState::Ok,
            results: count,
            ..Self::error(text)
        }
    }
}
