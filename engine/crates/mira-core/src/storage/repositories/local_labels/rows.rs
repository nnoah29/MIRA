//! Conversion des lignes de `local_labels` et erreurs communes du dépôt.

use mira_vocabulary::error::codes::{invalid_input, not_found};
use mira_vocabulary::{ErrorCategory, LocalLabel, LocalLabelId, MiraError};
use rusqlite::Row;

pub(super) const COLUMNS: &str = "id, name, color";

/// Lit une ligne dans l'ordre de `COLUMNS`
pub(super) fn label_from_row(row: &Row<'_>) -> rusqlite::Result<LocalLabel> {
    Ok(LocalLabel {
        id: LocalLabelId::from_raw(row.get::<_, String>(0)?),
        name: row.get(1)?,
        color: row.get(2)?,
    })
}

/// Clé d'unicité d'un nom : minuscules, espaces de bord retirés
pub(super) fn name_key(name: &str) -> String {
    name.trim().to_lowercase()
}

/// Erreur « libellé local inconnu »
pub(super) fn label_not_found() -> MiraError {
    MiraError::new(
        ErrorCategory::NotFound,
        not_found::LOCAL_LABEL,
        "libellé local inconnu",
    )
}

/// Erreur « fil inconnu »
pub(super) fn thread_not_found() -> MiraError {
    MiraError::new(ErrorCategory::NotFound, not_found::THREAD, "fil inconnu")
}

/// Erreur « nom déjà pris »
pub(super) fn name_taken() -> MiraError {
    MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::PARAMETER,
        "un libellé porte déjà ce nom",
    )
}
