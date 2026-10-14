//! Rafraîchissement silencieux du jeton d'accès d'un compte OAuth (7.7, S10), commun au rejeu et à la synchronisation.

use std::sync::Arc;

use mira_vocabulary::{AccountId, AuthMethodKind, MiraError, SecretKind};

use super::vault::CredentialVault;
use crate::accounts::require_account;
use crate::api::EngineConfig;
use crate::registry::resolve_account_provider;
use crate::storage::Database;

/// Rafraîchit le jeton OAuth du compte, toujours ou seulement s'il est expiré ; sans effet pour les autres méthodes
pub(crate) async fn renew_access_token(
    config: &EngineConfig,
    database: &Arc<Database>,
    account: &AccountId,
    always: bool,
) -> Result<(), MiraError> {
    let found = database.read(|tx| require_account(tx, account))?;
    if found.auth != AuthMethodKind::OAuth2 {
        return Ok(());
    }
    let vault = CredentialVault::new(Arc::clone(&config.secrets), Arc::clone(database));
    let expired = vault.access_token_expired(account, config.clock.now())?;
    if !always && !expired {
        return Ok(());
    }
    let provider = resolve_account_provider(&config.registry, &found.provider)?;
    let (settings, client_id) = config.oauth.resolve(&provider.descriptor())?;
    let seen = vault.load_secret(&found.profile, account, SecretKind::AccessToken)?;
    let secret = config.oauth.client_secret(&provider.descriptor());
    let refreshed = vault.refresh_access_token_if_stale_with_secret(
        &found.profile,
        account,
        &settings.token_url,
        (&client_id, secret.as_ref()),
        (config.clock.as_ref(), seen.as_ref()),
    );
    refreshed.await.map(|_| ())
}
