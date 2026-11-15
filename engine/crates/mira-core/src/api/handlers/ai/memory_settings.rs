//! Réglages de la mémoire : validés avant d'être enregistrés ; un changement arrête le serveur d'embeddings.

use mira_vocabulary::error::codes::invalid_input::AI_MEMORY_MODEL;
use mira_vocabulary::{ErrorCategory, MemoryEmbedder, MemorySettings, MiraError};

use super::memory_read::handle_get_memory_status;
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Refuse une connexion inconnue ou un nom de modèle vide pour le mode fournisseur
fn validate(engine: &Engine, settings: &MemorySettings) -> Result<(), MiraError> {
    let MemoryEmbedder::Provider { connection, model } = &settings.embedder else {
        return Ok(());
    };
    engine
        .database
        .read(|tx| tx.require_ai_connection(connection))?;
    if model.trim().is_empty() {
        return Err(MiraError::new(
            ErrorCategory::InvalidInput,
            AI_MEMORY_MODEL,
            "le modèle d'embeddings ne peut pas être vide",
        ));
    }
    Ok(())
}

/// Enregistre les réglages puis rend le statut ; le serveur d'embeddings est arrêté, il repartira au besoin
pub(crate) async fn handle_set_memory_settings(
    engine: &Engine,
    settings: MemorySettings,
) -> Result<Response, MiraError> {
    validate(engine, &settings)?;
    engine
        .database
        .write(|tx| tx.set_ai_memory_settings(&settings))?;
    engine.stop_memory_runtime().await;
    handle_get_memory_status(engine)
}
