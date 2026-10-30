//! Lectures sur les connexions d'IA.

use mira_vocabulary::{AiConnection, AiConnectionId, MiraError};
use rusqlite::OptionalExtension;

use super::errors::connection_not_found;
use super::rows::{COLUMNS, connection_from_row};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Relit une connexion (`has_key` toujours faux, rempli plus haut), ou rien si elle n'existe pas
    pub fn get_ai_connection(
        &self,
        id: &AiConnectionId,
    ) -> Result<Option<AiConnection>, MiraError> {
        self.sql()
            .query_row(
                &format!("SELECT {COLUMNS} FROM ai_connections WHERE id = ?1"),
                [id.as_str()],
                connection_from_row,
            )
            .optional()
            .map_err(storage_error)?
            .transpose()
    }

    /// Liste les connexions par libellé (sans tenir compte de la casse), puis identifiant
    pub fn list_ai_connections(&self) -> Result<Vec<AiConnection>, MiraError> {
        let mut statement = self
            .sql()
            .prepare(&format!("SELECT {COLUMNS} FROM ai_connections"))
            .map_err(storage_error)?;
        let rows = statement
            .query_map([], connection_from_row)
            .map_err(storage_error)?;
        let mut found = rows
            .collect::<Result<Result<Vec<_>, _>, _>>()
            .map_err(storage_error)??;
        found.sort_by_cached_key(|item| (item.label.to_lowercase(), item.id.as_str().to_string()));
        Ok(found)
    }

    /// Relit une connexion ou échoue avec `not_found.ai_connection`
    pub fn require_ai_connection(&self, id: &AiConnectionId) -> Result<AiConnection, MiraError> {
        self.get_ai_connection(id)?.ok_or_else(connection_not_found)
    }
}
