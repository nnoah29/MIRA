//! Ce qu'un appel d'outil a donné, avant cadrage : texte, état, sources et propositions pour l'application.

use mira_vocabulary::{
    ActionProposal, ContentKind, DraftProposal, MailCards, MemoryProposal, ToolState,
};

const DENIED_TEXT: &str =
    "accès refusé : l'utilisateur n'a pas autorisé ce type de contenu pour cette connexion";

/// Ce qu'un appel a donné, avant cadrage.
pub(super) struct Finished {
    pub(super) text: String,
    pub(super) state: ToolState,
    pub(super) results: u32,
    pub(super) sources: Vec<String>,
    pub(super) denied: Option<ContentKind>,
    pub(super) sent: Option<ContentKind>,
    pub(super) proposal: Option<DraftProposal>,
    pub(super) action: Option<ActionProposal>,
    pub(super) cards: Option<MailCards>,
    pub(super) memory: Option<MemoryProposal>,
}

impl Finished {
    /// Échec expliqué au modèle : rien n'a été lu.
    pub(super) fn error(text: &str) -> Self {
        Self {
            text: text.to_string(),
            state: ToolState::Error,
            results: 0,
            sources: Vec::new(),
            denied: None,
            sent: None,
            proposal: None,
            action: None,
            cards: None,
            memory: None,
        }
    }

    /// Refus faute d'autorisation de ce type : rien n'a été lu.
    pub(super) fn denied(kind: ContentKind) -> Self {
        Self {
            state: ToolState::Denied,
            denied: Some(kind),
            ..Self::error(DENIED_TEXT)
        }
    }
}
