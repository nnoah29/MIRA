//! Liste des brouillons d'un compte (lecture locale).

use mira_vocabulary::{AccountId, MiraError};

use crate::accounts::require_account;
use crate::api::contract::{Response, WriteResponse};
use crate::api::engine::Engine;

/// Rend les brouillons du compte
pub(crate) fn handle_list_drafts(
    engine: &Engine,
    account: &AccountId,
) -> Result<Response, MiraError> {
    let drafts = engine.database.read(|tx| {
        require_account(tx, account)?;
        tx.list_drafts(account)
    })?;
    Ok(Response::Write(WriteResponse::Drafts(drafts)))
}
