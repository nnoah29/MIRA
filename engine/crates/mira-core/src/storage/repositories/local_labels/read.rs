//! Lectures sur les libellés locaux.

use mira_vocabulary::{LocalLabel, LocalLabelId, MiraError};
use rusqlite::OptionalExtension;

use super::rows::{COLUMNS, label_from_row, label_not_found, name_key};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Relit un libellé, ou rien s'il n'existe pas
    pub fn get_local_label(&self, id: &LocalLabelId) -> Result<Option<LocalLabel>, MiraError> {
        self.sql()
            .query_row(
                &format!("SELECT {COLUMNS} FROM local_labels WHERE id = ?1"),
                [id.as_str()],
                label_from_row,
            )
            .optional()
            .map_err(storage_error)
    }

    /// Liste les libellés par nom (sans tenir compte de la casse), puis identifiant
    pub fn list_local_labels(&self) -> Result<Vec<LocalLabel>, MiraError> {
        let mut statement = self
            .sql()
            .prepare(&format!(
                "SELECT {COLUMNS} FROM local_labels ORDER BY name_key, id"
            ))
            .map_err(storage_error)?;
        statement
            .query_map([], label_from_row)
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)
    }

    /// Nombre de messages des conversations qui portent ce libellé
    pub fn local_label_message_count(&self, id: &LocalLabelId) -> Result<u64, MiraError> {
        self.sql()
            .query_row(
                "SELECT COUNT(*) FROM messages m \
                 JOIN threads t ON t.id = m.thread_id \
                 JOIN thread_local_labels tl ON tl.account_id = t.account_id AND tl.root_key = t.root_key \
                 WHERE tl.label_id = ?1",
                [id.as_str()],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| u64::try_from(count).unwrap_or(0))
            .map_err(storage_error)
    }

    /// Retrouve un libellé par son nom, casse et espaces de bord ignorés
    pub fn find_local_label_by_name(&self, name: &str) -> Result<Option<LocalLabel>, MiraError> {
        self.sql()
            .query_row(
                &format!("SELECT {COLUMNS} FROM local_labels WHERE name_key = ?1"),
                [name_key(name)],
                label_from_row,
            )
            .optional()
            .map_err(storage_error)
    }
}

/// Relit un libellé ou échoue avec `not_found.local_label`
pub fn require_label(tx: &Tx<'_>, id: &LocalLabelId) -> Result<LocalLabel, MiraError> {
    tx.get_local_label(id)?.ok_or_else(label_not_found)
}
