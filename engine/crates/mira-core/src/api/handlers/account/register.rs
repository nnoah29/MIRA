//! Enregistrement d'un compte validé : test de connexion, puis compte et identité en une transaction.

use mira_vocabulary::{Account, IdempotencyKey, IdentityId, MiraError};

use super::credential_login::StagedCredential;
use super::draft_plan::DraftPlan;
use crate::accounts::{NewAccountParts, build_account, build_default_identity, check_imap_login};
use crate::api::contract::AccountDraft;
use crate::api::engine::Engine;

/// Teste la connexion avec les identifiants rangés, puis enregistre le compte et son identité par défaut
pub(super) async fn register_account(
    engine: &Engine,
    draft: &AccountDraft,
    plan: &DraftPlan,
    staged: &StagedCredential,
    key: &IdempotencyKey,
) -> Result<Account, MiraError> {
    check_imap_login(&plan.servers.connect, &staged.login).await?;
    let parts = NewAccountParts {
        id: staged.account.clone(),
        identity_id: IdentityId::from_raw(engine.config.ids.next_id()),
        profile: engine.config.profile.clone(),
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
