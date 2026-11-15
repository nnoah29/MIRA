//! Écritures de la mémoire : retenir, modifier, oublier, tout oublier, réindexer ; oublier ne demande aucun embedder.

use mira_vocabulary::{MemoryKind, MemorySource, MiraError};

use crate::ai::memory::{forget_everything, forget_one};
use crate::api::contract::{AiResponse, Response};
use crate::api::engine::Engine;

/// Retient un texte à la demande de l'utilisateur ; mémoire désactivée : `incompatible_state.ai_memory_disabled`
pub(crate) async fn handle_remember_memory(
    engine: &Engine,
    text: &str,
    kind: MemoryKind,
) -> Result<Response, MiraError> {
    let store = engine.memory_store().await?;
    let record = store.remember(text, kind, MemorySource::User).await?;
    Ok(Response::Ai(AiResponse::Memory(record)))
}

/// Remplace le texte d'un souvenir ; inconnu : `not_found.ai_memory`
pub(crate) async fn handle_update_memory(
    engine: &Engine,
    id: &str,
    text: &str,
) -> Result<Response, MiraError> {
    let store = engine.memory_store().await?;
    let record = store.update(id, text).await?;
    Ok(Response::Ai(AiResponse::Memory(record)))
}

/// Oublie un souvenir, même mémoire désactivée ; inconnu : `not_found.ai_memory`
pub(crate) fn handle_forget_memory(engine: &Engine, id: &str) -> Result<Response, MiraError> {
    forget_one(&engine.database, id)?;
    Ok(Response::Ai(AiResponse::Done))
}

/// Oublie tous les souvenirs, même mémoire désactivée, et rend leur nombre
pub(crate) fn handle_forget_all_memories(engine: &Engine) -> Result<Response, MiraError> {
    let removed = forget_everything(&engine.database)?;
    Ok(Response::Ai(AiResponse::MemoryForgotten(removed)))
}

/// Recalcule les vecteurs des souvenirs périmés avec l'embedder courant
pub(crate) async fn handle_reindex_memories(engine: &Engine) -> Result<Response, MiraError> {
    let store = engine.memory_store().await?;
    let report = store.reindex().await?;
    Ok(Response::Ai(AiResponse::MemoryReindexed {
        done: report.done,
        failed: report.failed,
    }))
}
