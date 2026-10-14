//! Reprise d'un compte en pause : il repart hors ligne jusqu'à la prochaine connexion réussie.

use mira_vocabulary::{AccountId, AccountState, MiraError};

use crate::accounts::{change_account_state, require_active_account};
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Reprend un compte en pause et relance sa synchronisation ; un compte qui n'est pas en pause est rendu tel quel
pub(crate) fn handle_resume_account(
    engine: &Engine,
    id: &AccountId,
) -> Result<Response, MiraError> {
    let account = engine.database.read(|tx| require_active_account(tx, id))?;
    if account.state != AccountState::Paused {
        return Ok(Response::Account(account));
    }
    let resumed = change_account_state(
        &engine.database,
        &engine.events,
        account,
        AccountState::Offline,
    )?;
    if let Err(error) = engine.sync.start_account(&resumed) {
        tracing::warn!(code = %error.code, "synchronisation non relancée à la reprise");
    }
    Ok(Response::Account(resumed))
}
