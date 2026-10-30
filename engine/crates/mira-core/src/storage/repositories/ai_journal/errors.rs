//! Erreurs du dépôt du journal des envois d'IA.

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{ErrorCategory, MiraError};

/// Échoue avec `not_found.operation` si l'écriture n'a touché aucune ligne du journal
pub(super) fn ensure_changed(changed: usize) -> Result<(), MiraError> {
    match changed {
        0 => Err(MiraError::new(
            ErrorCategory::NotFound,
            not_found::OPERATION,
            "aucune ligne de journal pour cette opération",
        )),
        _ => Ok(()),
    }
}
