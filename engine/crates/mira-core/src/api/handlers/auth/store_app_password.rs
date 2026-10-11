//! Remplacement du mot de passe (d'application) d'un compte (7.7, S2).

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{Account, AccountId, AccountState, AuthMethodKind, ErrorCategory, MiraError};

use crate::accounts::{change_account_state, require_active_account};
use crate::api::contract::{Response, SecretInput};
use crate::api::engine::Engine;

/// Range le nouveau mot de passe dans le trousseau ; un compte à réauthentifier repasse hors ligne
pub(crate) fn handle_store_app_password(
    engine: &Engine,
    id: &AccountId,
    password: SecretInput,
) -> Result<Response, MiraError> {
    let account = engine.database.read(|tx| require_active_account(tx, id))?;
    require_password_account(&account, &password)?;
    engine
        .vault()
        .store_password(&account.profile, id, &password.into_secret_value())?;
    if account.state != AccountState::AuthRequired {
        return Ok(Response::Account(account));
    }
    let retried = change_account_state(
        &engine.database,
        &engine.events,
        account,
        AccountState::Offline,
    )?;
    Ok(Response::Account(retried))
}

/// Refuse un compte OAuth2 ou un mot de passe vide
fn require_password_account(account: &Account, password: &SecretInput) -> Result<(), MiraError> {
    let usable = account.auth != AuthMethodKind::OAuth2 && !password.is_empty();
    if usable {
        return Ok(());
    }
    Err(MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::PARAMETER,
        "ce compte n'utilise pas de mot de passe, ou le mot de passe est vide",
    ))
}
