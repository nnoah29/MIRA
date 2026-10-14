//! Statistiques de volume du miroir local.

use mira_vocabulary::MiraError;

use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Rend les volumes du miroir : boîtes, messages, sources, pièces jointes
pub(crate) fn handle_get_storage_stats(engine: &Engine) -> Result<Response, MiraError> {
    Ok(Response::StorageStats(
        engine.database.read(|tx| tx.storage_stats())?,
    ))
}
