//! Réindexation après un changement de modèle d'embeddings : les anciens vecteurs sont recalculés par lots.

use mira_vocabulary::error::codes::server::AI_BAD_RESPONSE;
use mira_vocabulary::{ErrorCategory, MiraError};

use super::store::MemoryStore;
use super::vectors::encode_vector;

const BATCH: u32 = 16;

impl MemoryStore {
    /// Calcule un vecteur pour un texte ; une réponse sans vecteur est refusée
    pub(super) async fn embed_one(&self, text: &str) -> Result<Vec<f32>, MiraError> {
        let texts = [text.to_string()];
        let mut vectors = self.embedder.embed(&texts).await?;
        vectors.pop().ok_or_else(|| {
            MiraError::new(
                ErrorCategory::Server,
                AI_BAD_RESPONSE,
                "réponse d'embeddings inutilisable : aucun vecteur",
            )
        })
    }

    /// Marque périmés les souvenirs d'un autre modèle puis les recalcule ; rend le nombre de souvenirs réindexés
    pub async fn reindex(&self) -> Result<u64, MiraError> {
        let model = self.embedder.model_id().to_string();
        self.db
            .write(|tx| tx.mark_ai_memories_stale_except(&model))?;
        let mut done = 0;
        loop {
            let batch = self.db.read(|tx| tx.stale_ai_memories(BATCH))?;
            if batch.is_empty() {
                return Ok(done);
            }
            let texts: Vec<String> = batch.iter().map(|m| m.text.clone()).collect();
            let vectors = self.embedder.embed(&texts).await?;
            for (record, vector) in batch.iter().zip(&vectors) {
                let dim = u32::try_from(vector.len()).unwrap_or(u32::MAX);
                let bytes = encode_vector(vector);
                self.db.write(|tx| {
                    tx.update_ai_memory_text(&record.id, &record.text, &bytes, dim, &model)
                })?;
                done += 1;
            }
        }
    }
}
