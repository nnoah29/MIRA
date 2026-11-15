//! Recherche par le sens : seuls les vecteurs du modèle courant et de bonne dimension sont comparés.

use mira_vocabulary::{MemoryHit, MemoryRecord, MiraError};

use super::ranking::top_k;
use super::store::MemoryStore;
use super::vectors::decode_vector;

const MAX_K: usize = 10;
const MIN_SCORE: f32 = 0.3;

impl MemoryStore {
    /// Les souvenirs les plus proches de la question, au plus `k` (plafonné à 10), du plus proche au moins proche
    pub async fn recall(&self, query: &str, k: usize) -> Result<Vec<MemoryHit>, MiraError> {
        let model = self.embedder.model_id().to_string();
        let stored = self.db.read(|tx| tx.ai_memory_vectors(&model))?;
        if stored.is_empty() || query.trim().is_empty() {
            return Ok(Vec::new());
        }
        let question = self.embed_one(query.trim()).await?;
        let mut records: Vec<MemoryRecord> = Vec::new();
        let mut items = Vec::new();
        for (record, bytes, dim) in stored {
            if let Some(vector) = decode_vector(&bytes, dim) {
                items.push((record.id.clone(), vector));
                records.push(record);
            }
        }
        let ranked = top_k(&question, items, k.min(MAX_K), MIN_SCORE);
        Ok(ranked
            .into_iter()
            .filter_map(|(id, score)| {
                let record = records.iter().find(|r| r.id == id)?.clone();
                Some(MemoryHit { record, score })
            })
            .collect())
    }
}
