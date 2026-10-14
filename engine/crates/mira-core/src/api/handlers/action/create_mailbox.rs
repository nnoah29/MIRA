//! Création d'une boîte (éventuellement sous un parent) : intention locale, serveur plus tard.

use mira_vocabulary::{AccountId, ActionKind, IdempotencyKey, MailboxId, MiraError};

use super::mailbox_submit::submit_mailbox_action;
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Met en file la création de la boîte et la fait apparaître tout de suite dans le miroir
pub(crate) fn handle_create_mailbox(
    engine: &Engine,
    account: AccountId,
    name: String,
    parent: Option<MailboxId>,
    key: IdempotencyKey,
) -> Result<Response, MiraError> {
    let kind = ActionKind::CreateMailbox { name, parent };
    submit_mailbox_action(engine, account, kind, key)
}
