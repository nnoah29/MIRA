//! Introspection du schéma : version, tables et volumes (diagnostic et tests d'invariants).

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{ErrorCategory, MiraError};

use super::errors::storage_error;
use super::tx::Tx;

impl Tx<'_> {
    /// Version du schéma enregistrée dans la base
    pub fn schema_version_on_disk(&self) -> i64 {
        self.sql()
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap_or(0)
    }

    /// Noms des tables du schéma (hors tables internes de SQLite et de FTS5)
    pub fn table_names(&self) -> Result<Vec<String>, MiraError> {
        let mut statement = self
            .sql()
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
            .map_err(storage_error)?;
        let rows = statement
            .query_map([], |row| row.get(0))
            .map_err(storage_error)?;
        rows.collect::<Result<Vec<String>, _>>()
            .map_err(storage_error)
    }

    /// Nombre de lignes d'une table existante du schéma
    pub fn table_row_count(&self, table: &str) -> Result<i64, MiraError> {
        if !self.table_names()?.iter().any(|name| name == table) {
            return Err(MiraError::new(
                ErrorCategory::InvalidInput,
                invalid_input::PARAMETER,
                "table inconnue",
            ));
        }
        let query = format!("SELECT COUNT(*) FROM \"{table}\"");
        self.sql()
            .query_row(&query, [], |row| row.get(0))
            .map_err(storage_error)
    }
}
