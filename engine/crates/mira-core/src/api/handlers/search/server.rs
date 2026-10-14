//! Recherche étendue au serveur : opération longue et annulable.

use mira_vocabulary::{AccountId, MiraError};

use crate::accounts::require_active_account;
use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::fetching::search_server;
use crate::operations::run_in_background;

/// Démarre la recherche côté serveur et rend la fiche de l'opération
pub(crate) fn handle_search_server(
    engine: &Engine,
    account: &AccountId,
    query: String,
) -> Result<Response, MiraError> {
    let record = engine
        .database
        .read(|tx| require_active_account(tx, account))?;
    let handle = engine.operations.start("search_server");
    let info = engine.operations.get(handle.id())?;
    let task = engine.fetch_task(handle.clone());
    let results = engine.server_searches.clone();
    run_in_background(handle, async move {
        let result = search_server(&task, &record, &query).await?;
        if !task.handle.is_cancelled() {
            results.keep(task.handle.id().clone(), result);
        }
        Ok(())
    });
    Ok(Response::Operation(info))
}
