//! Décision d'exécuter un outil : type de contenu connu, autorisé par la connexion, ou exempté sur la boucle locale.

use mira_vocabulary::ContentKind;

use crate::ai::tools::{is_proposal, kind_of};

/// Ce que le moteur fait d'un appel d'outil avant de lire quoi que ce soit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolDecision {
    Unknown,
    Denied(ContentKind),
    Run(ContentKind),
    Proposal,
}

/// Décide selon le type de contenu de l'outil : inconnu, refusé (type non autorisé), ou à exécuter.
pub fn decide_tool(
    local: bool,
    allowed: &[ContentKind],
    kind: Option<ContentKind>,
) -> ToolDecision {
    match kind {
        None => ToolDecision::Unknown,
        Some(kind) if local || allowed.contains(&kind) => ToolDecision::Run(kind),
        Some(kind) => ToolDecision::Denied(kind),
    }
}

/// Décide un appel par son nom : une proposition de brouillon n'envoie rien au service et ne demande aucun type, le reste suit le type de son résultat.
pub fn decide_call(local: bool, allowed: &[ContentKind], name: &str) -> ToolDecision {
    if is_proposal(name) {
        return ToolDecision::Proposal;
    }
    decide_tool(local, allowed, kind_of(name))
}
