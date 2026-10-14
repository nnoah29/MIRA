//! Suppression d'un compte (S14) : données locales et secrets seulement, jamais le serveur.

use mira_vocabulary::{AccountId, MiraError};

use crate::accounts::{erase_account, require_account};
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Marque le compte en suppression, interrompt sa synchronisation, abandonne ses flux OAuth, puis efface secrets et données
pub(crate) async fn handle_delete_account(
    engine: &Engine,
    id: &AccountId,
) -> Result<Response, MiraError> {
    let account = engine.database.read(|tx| require_account(tx, id))?;
    engine.database.write(|tx| tx.mark_account_deleting(id))?;
    engine.sync.stop_account(id).await;
    engine.oauth_flows.discard_account(id);
    erase_account(&engine.database, &engine.vault(), &engine.events, &account)?;
    Ok(Response::Done)
}
