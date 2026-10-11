//! Modification des réglages d'un compte.

use mira_vocabulary::{AccountId, EventKind, MiraError};

use super::changes::apply_changes;
use crate::accounts::require_active_account;
use crate::api::contract::{AccountChanges, Response};
use crate::api::engine::Engine;
use crate::events::EventSink;

/// Applique les modifications demandées, de façon atomique, puis émet `AccountUpdated`
pub(crate) fn handle_update_account(
    engine: &Engine,
    id: &AccountId,
    changes: AccountChanges,
) -> Result<Response, MiraError> {
    let updated = engine.database.write(|tx| {
        let mut account = require_active_account(tx, id)?;
        apply_changes(tx, &mut account, changes)?;
        tx.update_account(&account)?;
        Ok(account)
    })?;
    engine.events.emit(EventKind::AccountUpdated {
        account: id.clone(),
    })?;
    Ok(Response::Account(updated))
}
