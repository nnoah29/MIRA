//! Écritures sur les souvenirs : insertion, réindexation d'un texte, suppression, péremption par changement de modèle.

use mira_vocabulary::{MemoryRecord, MiraError};
use rusqlite::params;

use super::kinds::{kind_text, source_text};
use super::rows::unknown_memory;
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Ajoute un souvenir avec son vecteur brut (f32 little-endian) et la dimension de ce vecteur
    pub fn insert_ai_memory(
        &self,
        record: &MemoryRecord,
        embedding: &[u8],
        dim: u32,
    ) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "INSERT INTO ai_memories (id, text, kind, source, created_at, model_id, embedding, dim) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    record.id,
                    record.text,
                    kind_text(record.kind),
                    source_text(record.source),
                    record.created_at.as_millis(),
                    record.model_id,
                    embedding,
                    dim
                ],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Remplace le texte et le vecteur d'un souvenir et le rend de nouveau cherchable pour ce modèle
    pub fn update_ai_memory_text(
        &self,
        id: &str,
        text: &str,
        embedding: &[u8],
        dim: u32,
        model_id: &str,
    ) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE ai_memories SET text = ?2, embedding = ?3, dim = ?4, model_id = ?5, stale = 0 WHERE id = ?1",
                params![id, text, embedding, dim, model_id],
            )
            .map_err(storage_error)?;
        if changed == 0 {
            return Err(unknown_memory());
        }
        Ok(())
    }

    /// Supprime un souvenir ; inconnu : `not_found.ai_memory`
    pub fn delete_ai_memory(&self, id: &str) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute("DELETE FROM ai_memories WHERE id = ?1", [id])
            .map_err(storage_error)?;
        if changed == 0 {
            return Err(unknown_memory());
        }
        Ok(())
    }

    /// Supprime tous les souvenirs et rend leur nombre
    pub fn delete_all_ai_memories(&self) -> Result<u64, MiraError> {
        let changed = self
            .sql()
            .execute("DELETE FROM ai_memories", [])
            .map_err(storage_error)?;
        Ok(changed as u64)
    }

    /// Marque périmés les souvenirs indexés par un autre modèle que celui-ci et rend leur nombre
    pub fn mark_ai_memories_stale_except(&self, model_id: &str) -> Result<u64, MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE ai_memories SET stale = 1 WHERE stale = 0 AND (model_id IS NULL OR model_id <> ?1)",
                [model_id],
            )
            .map_err(storage_error)?;
        Ok(changed as u64)
    }
}
