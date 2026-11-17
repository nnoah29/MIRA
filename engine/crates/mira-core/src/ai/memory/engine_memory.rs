//! Mémoire de l'IA côté moteur : le service est construit à la demande selon les réglages, jamais en cache.

use std::sync::Arc;

use mira_vocabulary::error::codes::incompatible_state::{
    AI_LOCAL_UNSUPPORTED_PLATFORM, AI_MEMORY_DISABLED,
};
use mira_vocabulary::{ErrorCategory, MemoryEmbedder, MiraError};

use super::embedder::Embedder;
use super::local_embedder::LocalEmbedder;
use super::provider_embedder::provider_embedder;
use super::store::MemoryStore;
use crate::ai::local::{embedding_setup, local_descriptor};
use crate::api::Engine;

impl Engine {
    /// Service de mémoire selon les réglages ; désactivée : `incompatible_state.ai_memory_disabled`, sans rien lancer
    pub async fn memory_store(&self) -> Result<MemoryStore, MiraError> {
        let settings = self.database.read(|tx| tx.ai_memory_settings())?;
        if !settings.enabled {
            return Err(MiraError::new(
                ErrorCategory::IncompatibleState,
                AI_MEMORY_DISABLED,
                "la mémoire de l'IA est désactivée",
            ));
        }
        let embedder: Arc<dyn Embedder> = match &settings.embedder {
            MemoryEmbedder::Local => Arc::new(self.local_embedder()?),
            MemoryEmbedder::Provider { connection, model } => {
                Arc::new(provider_embedder(self, connection, model).await?)
            }
        };
        Ok(MemoryStore::new(
            self.database.clone(),
            embedder,
            self.config.clock.clone(),
            self.config.ids.clone(),
        ))
    }

    /// Arrête le serveur d'embeddings s'il tourne, par exemple après un changement de réglage
    pub async fn stop_memory_runtime(&self) {
        self.ai.memory.stop().await;
    }

    /// Embedder local : serveur dédié décrit par la fiche du runtime local
    fn local_embedder(&self) -> Result<LocalEmbedder, MiraError> {
        let unsupported = || {
            MiraError::new(
                ErrorCategory::IncompatibleState,
                AI_LOCAL_UNSUPPORTED_PLATFORM,
                "les embeddings locaux ne sont pas pris en charge",
            )
        };
        let descriptor = local_descriptor(self).ok_or_else(unsupported)?;
        Ok(LocalEmbedder::new(embedding_setup(self, &descriptor)?))
    }
}
