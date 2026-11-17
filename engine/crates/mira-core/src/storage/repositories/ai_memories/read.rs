//! Lecture des souvenirs : liste récente et vecteurs d'un modèle.

use mira_vocabulary::{MemoryRecord, MiraError};

use rusqlite::OptionalExtension;

use super::rows::{COLUMNS, record_from_row, unknown_memory};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Rend au plus `limit` souvenirs, les plus récents d'abord ; `after` (identifiant) reprend après cette ligne
    pub fn list_ai_memories(
        &self,
        limit: u32,
        after: Option<&str>,
    ) -> Result<Vec<MemoryRecord>, MiraError> {
        let mut statement = self
            .sql()
            .prepare(&format!(
                "SELECT {COLUMNS} FROM ai_memories \
                 WHERE ?1 IS NULL OR (created_at, rowid) < \
                 (SELECT created_at, rowid FROM ai_memories WHERE id = ?1) \
                 ORDER BY created_at DESC, rowid DESC LIMIT ?2"
            ))
            .map_err(storage_error)?;
        let rows = statement
            .query_map(rusqlite::params![after, limit], record_from_row)
            .map_err(storage_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(storage_error)
    }

    /// Rend les souvenirs indexés par ce modèle et non périmés, avec leur vecteur brut et sa dimension
    pub fn ai_memory_vectors(
        &self,
        model_id: &str,
    ) -> Result<Vec<(MemoryRecord, Vec<u8>, u32)>, MiraError> {
        let mut statement = self
            .sql()
            .prepare(&format!(
                "SELECT {COLUMNS}, embedding, dim FROM ai_memories \
                 WHERE model_id = ?1 AND stale = 0 AND embedding IS NOT NULL ORDER BY rowid"
            ))
            .map_err(storage_error)?;
        let rows = statement
            .query_map([model_id], |row| {
                Ok((record_from_row(row)?, row.get(6)?, row.get(7)?))
            })
            .map_err(storage_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(storage_error)
    }

    /// Rend au plus `limit` souvenirs périmés, les plus anciens d'abord, pour les réindexer
    pub fn stale_ai_memories(&self, limit: u32) -> Result<Vec<MemoryRecord>, MiraError> {
        let mut statement = self
            .sql()
            .prepare(&format!(
                "SELECT {COLUMNS} FROM ai_memories WHERE stale = 1 ORDER BY created_at, rowid LIMIT ?1"
            ))
            .map_err(storage_error)?;
        let rows = statement
            .query_map([limit], record_from_row)
            .map_err(storage_error)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(storage_error)
    }

    /// Rend un souvenir ; inconnu : `not_found.ai_memory`
    pub fn get_ai_memory(&self, id: &str) -> Result<MemoryRecord, MiraError> {
        self.sql()
            .query_row(
                &format!("SELECT {COLUMNS} FROM ai_memories WHERE id = ?1"),
                [id],
                record_from_row,
            )
            .optional()
            .map_err(storage_error)?
            .ok_or_else(unknown_memory)
    }

    /// Nombre de souvenirs et nombre de souvenirs à réindexer : périmés, ou d'un autre modèle que `current_model` s'il est connu
    pub fn count_ai_memories(&self, current_model: Option<&str>) -> Result<(u64, u64), MiraError> {
        self.sql()
            .query_row(
                "SELECT COUNT(*), COALESCE(SUM(stale = 1 OR (?1 IS NOT NULL AND (model_id IS NULL OR model_id <> ?1))), 0) FROM ai_memories",
                [current_model],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
            )
            .map(|(count, stale)| {
                (
                    u64::try_from(count).unwrap_or(0),
                    u64::try_from(stale).unwrap_or(0),
                )
            })
            .map_err(storage_error)
    }
}
