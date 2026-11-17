//! Lignes de `ai_memories` : colonnes lues dans l'ordre et conversion vers `MemoryRecord`.

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{ErrorCategory, MemoryRecord, MiraError, Timestamp};
use rusqlite::Row;
use rusqlite::types::Type;

use super::kinds::{parse_kind, parse_source};

pub(super) const COLUMNS: &str = "id, text, kind, source, created_at, model_id";

/// Lit un souvenir dans l'ordre de `COLUMNS`
pub(super) fn record_from_row(row: &Row<'_>) -> rusqlite::Result<MemoryRecord> {
    let kind: String = row.get(2)?;
    let source: String = row.get(3)?;
    Ok(MemoryRecord {
        id: row.get(0)?,
        text: row.get(1)?,
        kind: parse_kind(&kind).ok_or_else(|| unknown_text(2))?,
        source: parse_source(&source).ok_or_else(|| unknown_text(3))?,
        created_at: Timestamp::from_millis(row.get(4)?),
        model_id: row.get(5)?,
    })
}

/// Erreur de conversion d'une colonne de texte au contenu inattendu
fn unknown_text(column: usize) -> rusqlite::Error {
    rusqlite::Error::InvalidColumnType(column, "texte inattendu".into(), Type::Text)
}

/// Erreur d'un souvenir inconnu, sans aucun contenu
pub(super) fn unknown_memory() -> MiraError {
    MiraError::new(
        ErrorCategory::NotFound,
        not_found::AI_MEMORY,
        "souvenir introuvable",
    )
}
