//! Enregistrement d'un compte validé : secret, test de connexion, puis compte et identité en une transaction.

use mira_vocabulary::{Account, AccountId, IdempotencyKey, IdentityId, MiraError};

use super::credential_login::{password_login, secret_value_of};
use super::draft_plan::DraftPlan;
use crate::accounts::{NewAccountParts, build_account, build_default_identity, check_imap_login};
use crate::api::contract::{AccountDraft, CredentialInput};
use crate::api::engine::Engine;

/// Range le secret, teste la connexion, puis enregistre le compte et son identité par défaut
pub(super) async fn register_account(
    engine: &Engine,
    draft: &AccountDraft,
    plan: &DraftPlan,
    id: &AccountId,
    credential: CredentialInput,
    key: &IdempotencyKey,
) -> Result<Account, MiraError> {
    let secret = secret_value_of(credential);
    let profile = &engine.config.profile;
    engine.vault().store_password(profile, id, &secret)?;
    let login = password_login(&plan.address, secret);
    check_imap_login(&plan.servers.connect, &login).await?;
    let parts = NewAccountParts {
        id: id.clone(),
        identity_id: IdentityId::from_raw(engine.config.ids.next_id()),
        profile: profile.clone(),
        address: plan.address.clone(),
        display_name: draft.display_name.clone(),
        provider: plan.provider.descriptor().id,
        auth: plan.auth,
        servers: plan.servers.stored.clone(),
        now: engine.config.clock.now(),
    };
    persist(engine, &parts, key)
}

/// Écrit le compte, son identité par défaut et la clé d'idempotence, ou rien du tout
fn persist(
    engine: &Engine,
    parts: &NewAccountParts,
    key: &IdempotencyKey,
) -> Result<Account, MiraError> {
    let account = build_account(parts);
    let identity = build_default_identity(parts);
    engine.database.write(|tx| {
        tx.insert_account(&account)?;
        tx.insert_identity(&identity)?;
        tx.record_account_idempotency(key, parts.address.as_str(), &account.id)
    })?;
    Ok(account)
}
