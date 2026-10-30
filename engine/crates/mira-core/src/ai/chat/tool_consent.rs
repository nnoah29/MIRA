//! Décision d'exécuter un outil : type de contenu connu, autorisé par la connexion, ou exempté sur la boucle locale.

use mira_vocabulary::ContentKind;

/// Ce que le moteur fait d'un appel d'outil avant de lire quoi que ce soit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolDecision {
    Unknown,
    Denied(ContentKind),
    Run(ContentKind),
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
