//! Liste des identités d'envoi d'un compte (lecture locale).

use mira_vocabulary::{AccountId, MiraError};

use crate::accounts::require_account;
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Rend les identités du compte, l'identité par défaut en premier
pub(crate) fn handle_list_identities(
    engine: &Engine,
    account: &AccountId,
) -> Result<Response, MiraError> {
    let identities = engine.database.read(|tx| {
        require_account(tx, account)?;
        tx.list_identities(account)
    })?;
    Ok(Response::Identities(identities))
}
