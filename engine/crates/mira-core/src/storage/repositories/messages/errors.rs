//! Erreurs communes aux écritures sur les messages.

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{ErrorCategory, ErrorContext, MessageId, MiraError};

/// Erreur « message inconnu » portant l'identifiant visé
pub(crate) fn message_not_found(id: &MessageId) -> MiraError {
    MiraError::new(
        ErrorCategory::NotFound,
        not_found::MESSAGE,
        "message inconnu",
    )
    .with_context(ErrorContext {
        message: Some(id.clone()),
        ..ErrorContext::default()
    })
}

/// Échoue avec « message inconnu » si l'écriture n'a touché aucune ligne
pub(super) fn require_change(changed: usize, id: &MessageId) -> Result<(), MiraError> {
    if changed == 0 {
        return Err(message_not_found(id));
    }
    Ok(())
}
