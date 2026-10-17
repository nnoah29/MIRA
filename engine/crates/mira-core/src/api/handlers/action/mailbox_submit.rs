//! Mise en file d'une action sur une boîte : le compte est celui de la boîte visée.

use mira_vocabulary::{AccountId, ActionKind, IdempotencyKey, MailboxId, MiraError};

use crate::actions::{SubmitRequest, submit};
use crate::api::contract::{Response, WriteResponse};
use crate::api::engine::Engine;
use crate::storage::repositories::mailboxes::mailbox_not_found;

/// Compte propriétaire de la boîte, ou `not_found.mailbox`
pub(super) fn account_of_mailbox(
    engine: &Engine,
    mailbox: &MailboxId,
) -> Result<AccountId, MiraError> {
    engine.database.read(|tx| {
        tx.get_mailbox(mailbox)?
            .map(|found| found.account)
            .ok_or_else(|| mailbox_not_found(mailbox))
    })
}

/// Met l'action de boîte en file pour ce compte et rend son identifiant sans attendre le réseau
pub(super) fn submit_mailbox_action(
    engine: &Engine,
    account: AccountId,
    kind: ActionKind,
    key: IdempotencyKey,
) -> Result<Response, MiraError> {
    let quirks = engine.quirks_of(&account)?;
    let request = SubmitRequest {
        account,
        kind,
        targets: Vec::new(),
        key,
    };
    let action = submit(&engine.queue_context(), &quirks, request)?;
    engine.wake_replayers();
    Ok(Response::Write(WriteResponse::Action(action)))
}
