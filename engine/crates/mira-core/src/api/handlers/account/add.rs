//! Ajout d'un compte (7.5, S1–S3) : secret rangé, connexion testée, puis compte et identité enregistrés.

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{
    Account, AccountId, EmailAddress, ErrorCategory, EventKind, IdempotencyKey, MiraError,
};

use super::credential_login::{StagedCredential, stage_credential};
use super::draft_plan::plan_draft;
use super::register::register_account;
use crate::api::contract::{AccountDraft, CredentialInput, Response};
use crate::api::engine::Engine;
use crate::events::EventSink;

/// Ajoute le compte ; en cas d'échec rien n'est enregistré et le secret est supprimé
pub(crate) async fn handle_add_account(
    engine: &Engine,
    draft: AccountDraft,
    credential: CredentialInput,
    key: IdempotencyKey,
) -> Result<Response, MiraError> {
    let _serial = engine.account_lock.lock().await;
    if let Some(existing) = find_existing(engine, &key, &draft.address)? {
        return Ok(Response::Account(existing));
    }
    let plan = plan_draft(engine, &draft, &credential)?;
    let staged = stage_credential(engine, &plan, credential)?;
    let outcome = register_account(engine, &draft, &plan, &staged, &key).await;
    if let Some(flow) = &staged.flow {
        engine.oauth_flows.remove(flow);
    }
    let account = outcome.inspect_err(|_| discard_secrets(engine, &staged))?;
    engine.events.emit(EventKind::AccountAdded {
        account: account.id.clone(),
    })?;
    Ok(Response::Account(account))
}

/// Retrouve le compte déjà créé avec cette clé ; une autre adresse est un conflit
fn find_existing(
    engine: &Engine,
    key: &IdempotencyKey,
    address: &str,
) -> Result<Option<Account>, MiraError> {
    let address = EmailAddress::parse(address)?;
    engine.database.read(|tx| {
        let Some((id, stored)) = tx.account_for_idempotency_key(key)? else {
            return Ok(None);
        };
        if stored != address.as_str() {
            return Err(MiraError::new(
                ErrorCategory::InvalidInput,
                invalid_input::IDEMPOTENCY_CONFLICT,
                "clé d'idempotence déjà utilisée pour un autre compte",
            ));
        }
        tx.get_account(&id)
    })
}

/// Supprime les secrets déjà rangés pour un compte qui n'a pas pu être ajouté
fn discard_secrets(engine: &Engine, staged: &StagedCredential) {
    let id: &AccountId = &staged.account;
    if let Err(error) = engine.vault().delete_all(&engine.config.profile, id) {
        tracing::warn!(code = %error.code, "secrets d'un compte refusé non supprimés");
    }
}
