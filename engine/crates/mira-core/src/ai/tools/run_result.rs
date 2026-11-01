//! Résultat d'une exécution d'outil : texte rendu, type de contenu, état, nombre de résultats et sources.

use mira_vocabulary::{ContentKind, DraftProposal, ToolState};

/// Ce que l'outil renvoie au modèle, et ce que l'app peut en montrer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolRun {
    pub text: String,
    pub kind: ContentKind,
    pub state: ToolState,
    pub results: u32,
    pub sources: Vec<String>,
    pub proposal: Option<DraftProposal>,
}

impl ToolRun {
    /// Échec expliqué au modèle, sans contenu de mail ni détail interne.
    pub fn error(text: String) -> Self {
        Self {
            text,
            kind: ContentKind::ToolResult,
            state: ToolState::Error,
            results: 0,
            sources: Vec::new(),
            proposal: None,
        }
    }

    /// Liste réussie : un résultat par source.
    pub fn listed(text: String, kind: ContentKind, sources: Vec<String>) -> Self {
        Self {
            text,
            kind,
            state: ToolState::Ok,
            results: u32::try_from(sources.len()).unwrap_or(u32::MAX),
            sources,
            proposal: None,
        }
    }

    /// Proposition de brouillon acceptée : la phrase fixe renvoyée au modèle et la proposition pour l'application.
    pub fn proposed(text: String, proposal: DraftProposal) -> Self {
        Self {
            text,
            kind: ContentKind::ToolResult,
            state: ToolState::Ok,
            results: 1,
            sources: Vec::new(),
            proposal: Some(proposal),
        }
    }
}
