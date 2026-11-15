//! Résultat d'une exécution d'outil : texte rendu, type de contenu, état, nombre de résultats et sources.

use mira_vocabulary::{ActionProposal, ContentKind, DraftProposal, MailCards, ToolState};

/// Ce que l'outil renvoie au modèle, et ce que l'app peut en montrer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolRun {
    pub text: String,
    pub kind: ContentKind,
    pub state: ToolState,
    pub results: u32,
    pub sources: Vec<String>,
    pub proposal: Option<DraftProposal>,
    pub action: Option<ActionProposal>,
    pub cards: Option<MailCards>,
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
            action: None,
            cards: None,
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
            action: None,
            cards: None,
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
            action: None,
            cards: None,
        }
    }

    /// Action sur la boîte proposée : la phrase fixe renvoyée au modèle et la proposition pour l'application.
    pub fn action_proposed(text: String, action: ActionProposal) -> Self {
        Self {
            text,
            kind: ContentKind::ToolResult,
            state: ToolState::Ok,
            results: 1,
            sources: Vec::new(),
            proposal: None,
            action: Some(action),
            cards: None,
        }
    }

    /// Cartes de messages montrées : la phrase fixe renvoyée au modèle et les cartes pour l'application.
    pub fn cards_shown(text: String, cards: MailCards) -> Self {
        Self {
            text,
            kind: ContentKind::ToolResult,
            state: ToolState::Ok,
            results: u32::try_from(cards.cards.len()).unwrap_or(u32::MAX),
            sources: Vec::new(),
            proposal: None,
            action: None,
            cards: Some(cards),
        }
    }
}
