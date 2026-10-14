//! Mise en pause de la synchronisation d'un compte : la tâche s'arrête, le compte passe en pause.

use mira_vocabulary::{AccountId, MiraError};

use super::status_view::status_of;
use crate::api::contract::{Response, SyncResponse};
use crate::api::engine::Engine;
use crate::api::handlers::account::pause::pause_account;

/// Arrête la synchronisation du compte et rend son état ; reprise par `ResumeAccount`
pub(crate) async fn handle_pause_sync(
    engine: &Engine,
    id: &AccountId,
) -> Result<Response, MiraError> {
    let paused = pause_account(engine, id).await?;
    let status = status_of(engine, &paused)?;
    Ok(Response::Sync(SyncResponse::Paused(status)))
}
