//! Mises à jour des seuls vecteurs : sous condition de texte, et péremption par changement de dimension.

use mira_vocabulary::MiraError;
use rusqlite::params;

use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Remplace le vecteur seulement si le texte est toujours celui qui a été calculé ; rend `false` sinon ou si le souvenir a disparu
    pub fn update_ai_memory_vector(
        &self,
        id: &str,
        text: &str,
        embedding: &[u8],
        dim: u32,
        model_id: &str,
    ) -> Result<bool, MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE ai_memories SET embedding = ?3, dim = ?4, model_id = ?5, stale = 0 WHERE id = ?1 AND text = ?2",
                params![id, text, embedding, dim, model_id],
            )
            .map_err(storage_error)?;
        Ok(changed > 0)
    }

    /// Marque périmés les souvenirs de ce modèle dont le vecteur n'a pas cette dimension et rend leur nombre
    pub fn mark_ai_memories_stale_other_dim(
        &self,
        model_id: &str,
        dim: u32,
    ) -> Result<u64, MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE ai_memories SET stale = 1 WHERE stale = 0 AND model_id = ?1 AND (dim IS NULL OR dim <> ?2)",
                params![model_id, dim],
            )
            .map_err(storage_error)?;
        Ok(changed as u64)
    }
}
