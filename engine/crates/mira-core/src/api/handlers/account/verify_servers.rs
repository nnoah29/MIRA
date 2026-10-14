//! Vérification de nouveaux serveurs avant de les enregistrer : connexion avec le secret rangé.

use mira_vocabulary::{AccountId, MiraError, ServerSettings};

use crate::accounts::{check_imap_login, imap_login_of, require_active_account, resolve_servers};
use crate::api::engine::Engine;
use crate::auth::renewal::renew_access_token;
use crate::registry::resolve_account_provider;

/// Refuse des serveurs différents de ceux du compte tant qu'une connexion n'y réussit pas
pub(super) async fn verify_new_servers(
    engine: &Engine,
    id: &AccountId,
    requested: &ServerSettings,
) -> Result<(), MiraError> {
    let account = engine.database.read(|tx| require_active_account(tx, id))?;
    if account.servers.as_ref() == Some(requested) {
        return Ok(());
    }
    let provider = resolve_account_provider(&engine.config.registry, &account.provider)?;
    let servers = resolve_servers(
        &engine.config.network,
        provider.as_ref(),
        &account.address,
        Some(requested.clone()),
    );
    renew_access_token(&engine.config, &engine.database, id, false).await?;
    let user = account.address.as_str();
    let secret = engine
        .vault()
        .load_login(&account.profile, id, account.auth, user)?;
    check_imap_login(&servers.connect, &imap_login_of(secret)).await
}
