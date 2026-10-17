//! Mise en pause d'un compte : la synchronisation s'arrête, les données restent lisibles.

use mira_vocabulary::{Account, AccountId, AccountState, MiraError};

use crate::accounts::{change_account_state, require_active_account};
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Arrête la tâche de synchronisation puis met le compte en pause ; déjà en pause, il est rendu tel quel
pub(crate) async fn pause_account(engine: &Engine, id: &AccountId) -> Result<Account, MiraError> {
    let account = engine.database.read(|tx| require_active_account(tx, id))?;
    if account.state == AccountState::Paused {
        return Ok(account);
    }
    engine.sync.stop_account(id).await;
    change_account_state(
        &engine.database,
        &engine.events,
        account,
        AccountState::Paused,
    )
}

/// Met le compte en pause ; déjà en pause, il est rendu tel quel sans événement
pub(crate) async fn handle_pause_account(
    engine: &Engine,
    id: &AccountId,
) -> Result<Response, MiraError> {
    Ok(Response::Account(pause_account(engine, id).await?))
}
