//! Suppression d'un compte (S14) : données locales et secrets seulement, jamais le serveur.

use mira_vocabulary::{AccountId, EventKind, MiraError};

use crate::accounts::require_account;
use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::events::EventSink;

/// Marque le compte en suppression, efface ses secrets puis ses données, et émet `AccountDeleted`
pub(crate) fn handle_delete_account(
    engine: &Engine,
    id: &AccountId,
) -> Result<Response, MiraError> {
    let account = engine.database.read(|tx| require_account(tx, id))?;
    engine.database.write(|tx| tx.mark_account_deleting(id))?;
    engine.vault().delete_all(&account.profile, id)?;
    engine.database.write(|tx| tx.delete_account(id))?;
    engine.events.emit(EventKind::AccountDeleted {
        account: id.clone(),
    })?;
    Ok(Response::Done)
}
