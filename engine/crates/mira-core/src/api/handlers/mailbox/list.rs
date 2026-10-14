//! Liste des boîtes d'un compte avec leurs compteurs (lecture locale).

use mira_vocabulary::{AccountId, MiraError};

use crate::accounts::require_account;
use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::reading::mailboxes_with_counters;

/// Rend les boîtes du compte, compteurs recalculés depuis les apparitions
pub(crate) fn handle_list_mailboxes(
    engine: &Engine,
    account: &AccountId,
) -> Result<Response, MiraError> {
    engine.database.read(|tx| require_account(tx, account))?;
    let mailboxes = mailboxes_with_counters(&engine.database, account)?;
    Ok(Response::Mailboxes(mailboxes))
}
