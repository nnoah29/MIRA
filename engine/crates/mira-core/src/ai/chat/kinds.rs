//! Types de contenu d'une requête : autorisation par connexion et liste distincte pour le journal.

use mira_vocabulary::{AiConnection, AiError, ContentKind, ContextPart, MiraError};

/// Refuse un type absent de la liste de la connexion ; la boucle locale est exemptée.
pub(super) fn check_kinds(
    connection: &AiConnection,
    parts: &[ContextPart],
    local: bool,
) -> Result<(), MiraError> {
    let allowed = |part: &ContextPart| connection.allowed_kinds.contains(&part.kind);
    match local || parts.iter().all(allowed) {
        true => Ok(()),
        false => Err(AiError::ContentNotAllowed.into()),
    }
}

/// Types présents, chacun une fois, dans l'ordre de leur première apparition.
pub(super) fn distinct_kinds(parts: &[ContextPart]) -> Vec<ContentKind> {
    let mut kinds = Vec::new();
    for part in parts {
        if !kinds.contains(&part.kind) {
            kinds.push(part.kind);
        }
    }
    kinds
}
