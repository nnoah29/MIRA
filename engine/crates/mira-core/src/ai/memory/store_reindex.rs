//! Réindexation après un changement de modèle : les anciens vecteurs sont recalculés par lots, sans jamais boucler.

use mira_vocabulary::error::codes::server::AI_BAD_RESPONSE;
use mira_vocabulary::{ErrorCategory, MemoryRecord, MiraError};

use super::store::MemoryStore;
use super::vectors::encode_vector;

const BATCH: u32 = 16;

/// Bilan d'une réindexation : souvenirs recalculés et souvenirs que le modèle a refusés.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReindexReport {
    pub done: u64,
    pub failed: u64,
}

/// Erreur d'une réponse d'embeddings inutilisable, sans aucun contenu
fn unusable(reason: &str) -> MiraError {
    MiraError::new(
        ErrorCategory::Server,
        AI_BAD_RESPONSE,
        format!("réponse d'embeddings inutilisable : {reason}"),
    )
}

impl MemoryStore {
    /// Calcule les vecteurs et vérifie : un par texte, non vides, valeurs finies
    pub(super) async fn embed_checked(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, MiraError> {
        let vectors = self.embedder.embed(texts).await?;
        let sound = vectors
            .iter()
            .all(|v| !v.is_empty() && v.iter().all(|x| x.is_finite()));
        if vectors.len() != texts.len() || !sound {
            return Err(unusable("vecteurs invalides"));
        }
        Ok(vectors)
    }

    /// Calcule le vecteur d'un seul texte
    pub(super) async fn embed_one(&self, text: &str) -> Result<Vec<f32>, MiraError> {
        let mut vectors = self.embed_checked(&[text.to_string()]).await?;
        vectors.pop().ok_or_else(|| unusable("aucun vecteur"))
    }

    /// Marque périmés les souvenirs d'un autre modèle puis les recalcule ; un texte refusé est compté et laissé de côté
    pub async fn reindex(&self) -> Result<ReindexReport, MiraError> {
        let model = self.embedder.model_id().to_string();
        self.db
            .write(|tx| tx.mark_ai_memories_stale_except(&model))?;
        let mut report = ReindexReport::default();
        let mut skipped: Vec<String> = Vec::new();
        loop {
            let limit = BATCH + u32::try_from(skipped.len()).unwrap_or(0);
            let listed = self.db.read(|tx| tx.stale_ai_memories(limit))?;
            let batch: Vec<MemoryRecord> = listed
                .into_iter()
                .filter(|m| !skipped.contains(&m.id))
                .take(BATCH as usize)
                .collect();
            if batch.is_empty() {
                return Ok(report);
            }
            self.reindex_batch(&batch, &model, &mut report, &mut skipped)
                .await?;
        }
    }

    /// Recalcule un lot ; s'il échoue, reprend texte par texte pour isoler celui que le modèle refuse
    async fn reindex_batch(
        &self,
        batch: &[MemoryRecord],
        model: &str,
        report: &mut ReindexReport,
        skipped: &mut Vec<String>,
    ) -> Result<(), MiraError> {
        let texts: Vec<String> = batch.iter().map(|m| m.text.clone()).collect();
        let together = self.embed_checked(&texts).await;
        for (i, record) in batch.iter().enumerate() {
            let vector = match &together {
                Ok(all) => Ok(all[i].clone()),
                Err(_) => self.embed_one(&record.text).await,
            };
            match vector {
                Ok(vector) => self.store_vector(record, &vector, model, report)?,
                Err(_) => {
                    report.failed += 1;
                    skipped.push(record.id.clone());
                }
            }
        }
        Ok(())
    }

    /// Écrit le vecteur calculé ; ignoré sans erreur si le souvenir a été oublié ou modifié entre-temps
    fn store_vector(
        &self,
        record: &MemoryRecord,
        vector: &[f32],
        model: &str,
        report: &mut ReindexReport,
    ) -> Result<(), MiraError> {
        let dim = u32::try_from(vector.len()).unwrap_or(u32::MAX);
        let bytes = encode_vector(vector);
        let stored = self
            .db
            .write(|tx| tx.update_ai_memory_vector(&record.id, &record.text, &bytes, dim, model))?;
        report.done += u64::from(stored);
        Ok(())
    }
}
