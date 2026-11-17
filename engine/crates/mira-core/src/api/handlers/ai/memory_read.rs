//! Lectures de la mémoire : statut, liste et recherche ; la liste ne demande ni embedder ni mémoire activée.

use mira_vocabulary::{MemoryEmbedder, MemoryMatch, MemoryStatus, MiraError};

use crate::ai::local::embedding_installed;
use crate::ai::memory::current_model_id;
use crate::api::contract::{AiResponse, Response};
use crate::api::engine::Engine;

const MAX_PAGE: u32 = 200;

/// Réglages, présence du modèle local, nombre de souvenirs et de souvenirs à réindexer
pub(crate) fn handle_get_memory_status(engine: &Engine) -> Result<Response, MiraError> {
    let settings = engine.database.read(|tx| tx.ai_memory_settings())?;
    let current = current_model_id(engine, &settings);
    let (count, stale) = engine
        .database
        .read(|tx| tx.count_ai_memories(current.as_deref()))?;
    let model_installed = match settings.embedder {
        MemoryEmbedder::Local => embedding_installed(engine),
        MemoryEmbedder::Provider { .. } => true,
    };
    let status = MemoryStatus {
        settings,
        model_installed,
        count,
        stale,
    };
    Ok(Response::Ai(AiResponse::MemoryStatus(status)))
}

/// Liste les souvenirs, les plus récents d'abord ; `after` reprend après ce souvenir
pub(crate) fn handle_list_memories(
    engine: &Engine,
    limit: u32,
    after: Option<&str>,
) -> Result<Response, MiraError> {
    let limit = limit.clamp(1, MAX_PAGE);
    let items = engine
        .database
        .read(|tx| tx.list_ai_memories(limit, after))?;
    Ok(Response::Ai(AiResponse::Memories(items)))
}

/// Cherche par le sens ; mémoire désactivée : `incompatible_state.ai_memory_disabled`
pub(crate) async fn handle_search_memories(
    engine: &Engine,
    query: &str,
    limit: u32,
) -> Result<Response, MiraError> {
    let store = engine.memory_store().await?;
    let hits = store.recall(query, limit as usize).await?;
    let matches = hits.into_iter().map(MemoryMatch::from).collect();
    Ok(Response::Ai(AiResponse::MemoryMatches(matches)))
}
