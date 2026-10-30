//! Lecture du journal des envois d'IA, de la plus récente à la plus ancienne.

use mira_vocabulary::{AiJournalEntry, MiraError};

use super::rows::{COLUMNS, entry_from_row};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Rend au plus `limit` lignes, les plus récentes d'abord ; `before` (identifiant) reprend après cette ligne
    pub fn list_ai_journal(
        &self,
        limit: u32,
        before: Option<&str>,
    ) -> Result<Vec<AiJournalEntry>, MiraError> {
        let mut statement = self
            .sql()
            .prepare(&format!(
                "SELECT {COLUMNS} FROM ai_journal \
                 WHERE ?1 IS NULL OR (at, rowid) < (SELECT at, rowid FROM ai_journal WHERE id = ?1) \
                 ORDER BY at DESC, rowid DESC LIMIT ?2"
            ))
            .map_err(storage_error)?;
        let rows = statement
            .query_map(rusqlite::params![before, limit], entry_from_row)
            .map_err(storage_error)?;
        rows.collect::<Result<Result<Vec<_>, _>, _>>()
            .map_err(storage_error)?
    }
}
