//! Recherche par le sens : seuls les vecteurs du modèle courant et de la dimension de la question sont comparés.

use mira_vocabulary::{MemoryHit, MemoryRecord, MiraError};

use super::guard::MAX_CHARS;
use super::ranking::top_k;
use super::store::MemoryStore;
use super::vectors::decode_vector;

const MAX_K: usize = 10;
const MIN_SCORE: f32 = 0.3;

impl MemoryStore {
    /// Les souvenirs les plus proches de la question (tronquée à 1 000 caractères), au plus `k` plafonné à 10
    pub async fn recall(&self, query: &str, k: usize) -> Result<Vec<MemoryHit>, MiraError> {
        let model = self.embedder.model_id().to_string();
        let stored = self.db.read(|tx| tx.ai_memory_vectors(&model))?;
        let question: String = query.trim().chars().take(MAX_CHARS).collect();
        if stored.is_empty() || question.is_empty() || k == 0 {
            return Ok(Vec::new());
        }
        let vector = self.embed_one(&question).await?;
        let dim = u32::try_from(vector.len()).unwrap_or(u32::MAX);
        self.db
            .write(|tx| tx.mark_ai_memories_stale_other_dim(&model, dim))?;
        let mut records: Vec<MemoryRecord> = Vec::new();
        let mut items = Vec::new();
        for (record, bytes, stored_dim) in stored.into_iter().filter(|(_, _, d)| *d == dim) {
            if let Some(found) = decode_vector(&bytes, stored_dim) {
                items.push((record.id.clone(), found));
                records.push(record);
            }
        }
        let ranked = top_k(&vector, items, k.min(MAX_K), MIN_SCORE);
        Ok(ranked
            .into_iter()
            .filter_map(|(id, score)| {
                let record = records.iter().find(|r| r.id == id)?.clone();
                Some(MemoryHit { record, score })
            })
            .collect())
    }
}
