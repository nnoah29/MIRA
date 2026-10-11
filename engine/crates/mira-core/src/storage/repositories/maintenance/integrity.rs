//! Contrôle d'intégrité de la base SQLite.

use mira_vocabulary::MiraError;

use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Dit si `PRAGMA integrity_check` ne signale aucun défaut
    pub fn integrity_check(&self) -> Result<bool, MiraError> {
        let verdict: String = self
            .sql()
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .map_err(storage_error)?;
        Ok(verdict == "ok")
    }
}
