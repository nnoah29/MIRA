//! Mise en pause d'un compte : la synchronisation s'arrête, les données restent lisibles.

use mira_vocabulary::{AccountId, AccountState, MiraError};

use crate::accounts::{change_account_state, require_active_account};
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Met le compte en pause ; déjà en pause, il est rendu tel quel sans événement
pub(crate) fn handle_pause_account(engine: &Engine, id: &AccountId) -> Result<Response, MiraError> {
    let account = engine.database.read(|tx| require_active_account(tx, id))?;
    if account.state == AccountState::Paused {
        return Ok(Response::Account(account));
    }
    let paused = change_account_state(
        &engine.database,
        &engine.events,
        account,
        AccountState::Paused,
    )?;
    Ok(Response::Account(paused))
}
