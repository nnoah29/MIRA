//! Liste des actions d'un compte, dans l'ordre de la file (lecture locale).

use mira_vocabulary::{AccountId, ActionState, MiraError};

use crate::accounts::require_account;
use crate::api::contract::{Response, WriteResponse};
use crate::api::engine::Engine;

/// Rend les actions du compte dans ces états ; sans état demandé, toutes les actions
pub(crate) fn handle_list_actions(
    engine: &Engine,
    account: &AccountId,
    states: &[ActionState],
) -> Result<Response, MiraError> {
    let actions = engine.database.read(|tx| {
        require_account(tx, account)?;
        tx.list_actions(account, states)
    })?;
    Ok(Response::Write(WriteResponse::QueuedActions(actions)))
}
