//! Modification du texte d'un souvenir : mêmes garde-fous qu'à l'ajout, vecteur recalculé avant toute écriture.

use mira_vocabulary::{MemoryRecord, MiraError};

use super::guard::checked_text;
use super::store::MemoryStore;
use super::vectors::encode_vector;

impl MemoryStore {
    /// Remplace le texte d'un souvenir et son vecteur ; inconnu : `not_found.ai_memory`
    pub async fn update(&self, id: &str, text: &str) -> Result<MemoryRecord, MiraError> {
        let text = checked_text(text)?;
        let vector = self.embed_one(&text).await?;
        let dim = u32::try_from(vector.len()).unwrap_or(u32::MAX);
        let bytes = encode_vector(&vector);
        let model = self.embedder.model_id().to_string();
        self.db.write(|tx| {
            tx.update_ai_memory_text(id, &text, &bytes, dim, &model)?;
            tx.get_ai_memory(id)
        })
    }
}
