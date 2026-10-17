//! État de la synchronisation de chaque compte (12.4) : lecture locale uniquement.

use mira_vocabulary::MiraError;

use super::status_view::status_of;
use crate::api::contract::{Response, SyncResponse};
use crate::api::engine::Engine;

/// Rend l'état de synchronisation de tous les comptes du profil non supprimés
pub(crate) fn handle_get_sync_status(engine: &Engine) -> Result<Response, MiraError> {
    let accounts = engine.database.read(|tx| {
        let mut kept = Vec::new();
        for account in tx.list_accounts()? {
            if !tx.account_is_deleting(&account.id)? {
                kept.push(account);
            }
        }
        Ok(kept)
    })?;
    let statuses = accounts
        .iter()
        .map(|account| status_of(engine, account))
        .collect::<Result<Vec<_>, MiraError>>()?;
    Ok(Response::Sync(SyncResponse::Status(statuses)))
}
