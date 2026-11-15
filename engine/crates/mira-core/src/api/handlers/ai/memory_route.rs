//! Routage des requêtes de la mémoire de l'IA : une requête, un gestionnaire.

use mira_vocabulary::MiraError;

use super::memory_install::handle_install_memory_model as install_memory;
use super::memory_read::handle_search_memories as search;
use super::memory_read::{handle_get_memory_status as status, handle_list_memories as list};
use super::memory_settings::handle_set_memory_settings as set_settings;
use super::memory_write::handle_update_memory as update;
use super::memory_write::{
    handle_forget_all_memories as forget_all, handle_forget_memory as forget,
};
use super::memory_write::{handle_reindex_memories as reindex, handle_remember_memory as remember};
use crate::api::contract::{AiRequest as R, Response};
use crate::api::engine::Engine;

/// Résultat d'une requête du domaine
type Answer = Result<Response, MiraError>;

/// Route les requêtes de la mémoire
#[rustfmt::skip]
pub(super) async fn route_memory(engine: &Engine, request: R) -> Answer {
    match request {
        R::InstallMemoryModel => install_memory(engine),
        R::GetMemoryStatus => status(engine),
        R::SetMemorySettings { settings } => set_settings(engine, settings).await,
        R::ListMemories { limit, after } => list(engine, limit, after.as_deref()),
        R::SearchMemories { query, limit } => search(engine, &query, limit).await,
        R::RememberMemory { text, kind } => remember(engine, &text, kind).await,
        R::UpdateMemory { id, text } => update(engine, &id, &text).await,
        R::ForgetMemory { id } => forget(engine, &id),
        R::ForgetAllMemories => forget_all(engine),
        R::ReindexMemories => reindex(engine).await,
        _ => Err(super::unexpected()),
    }
}
