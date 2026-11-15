//! Service de mémoire de l'IA : retenir, oublier, avec des vecteurs calculés hors transaction.

use std::sync::Arc;

use mira_vocabulary::{MemoryKind, MemoryRecord, MemorySource, MiraError};

use super::embedder::Embedder;
use super::forgetting::{forget_everything, forget_one};
use super::guard::checked_text;
use super::vectors::encode_vector;
use crate::ports::{Clock, IdGenerator};
use crate::storage::Database;

/// Souvenirs durables de l'utilisateur, indexés par un embedder.
pub struct MemoryStore {
    pub(super) db: Arc<Database>,
    pub(super) embedder: Arc<dyn Embedder>,
    clock: Arc<dyn Clock>,
    ids: Arc<dyn IdGenerator>,
}

impl MemoryStore {
    /// Branche le service sur la base, l'embedder, l'horloge et le générateur d'identifiants
    pub fn new(
        db: Arc<Database>,
        embedder: Arc<dyn Embedder>,
        clock: Arc<dyn Clock>,
        ids: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            db,
            embedder,
            clock,
            ids,
        }
    }

    /// Enregistre un souvenir après contrôle du texte ; l'embedder est appelé avant toute écriture
    pub async fn remember(
        &self,
        text: &str,
        kind: MemoryKind,
        source: MemorySource,
    ) -> Result<MemoryRecord, MiraError> {
        let text = checked_text(text)?;
        let vector = self.embed_one(&text).await?;
        let record = MemoryRecord {
            id: self.ids.next_id(),
            text,
            kind,
            source,
            created_at: self.clock.now(),
            model_id: Some(self.embedder.model_id().to_string()),
        };
        let dim = u32::try_from(vector.len()).unwrap_or(u32::MAX);
        let bytes = encode_vector(&vector);
        self.db
            .write(|tx| tx.insert_ai_memory(&record, &bytes, dim))?;
        Ok(record)
    }

    /// Oublie un souvenir ; inconnu : `not_found.ai_memory`
    pub async fn forget(&self, id: &str) -> Result<(), MiraError> {
        forget_one(&self.db, id)
    }

    /// Oublie tout et rend le nombre de souvenirs effacés
    pub async fn forget_all(&self) -> Result<u64, MiraError> {
        forget_everything(&self.db)
    }
}

impl std::fmt::Debug for MemoryStore {
    /// Ne montre que le modèle d'embeddings, jamais le contenu des souvenirs
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MemoryStore")
            .field("model", &self.embedder.model_id())
            .finish_non_exhaustive()
    }
}
