//! Liste des comptes du profil (lecture locale, invariant 27).

use mira_vocabulary::MiraError;

use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Rend tous les comptes, du plus ancien au plus récent
pub(crate) fn handle_list_accounts(engine: &Engine) -> Result<Response, MiraError> {
    let accounts = engine.database.read(|tx| tx.list_accounts())?;
    Ok(Response::Accounts(accounts))
}
