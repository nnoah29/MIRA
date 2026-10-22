//! Lectures sur les libellés locaux.

use mira_vocabulary::{LocalLabel, LocalLabelId, MiraError};
use rusqlite::OptionalExtension;

use super::rows::{COLUMNS, label_from_row};
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
}
