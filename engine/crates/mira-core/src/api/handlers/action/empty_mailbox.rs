//! Vidage d'une boîte (corbeille ou spam).

use mira_vocabulary::{ActionKind, IdempotencyKey, MailboxId, MiraError};

use super::mailbox_submit::{account_of_mailbox, submit_mailbox_action};
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Met en file le vidage de la boîte
pub(crate) fn handle_empty_mailbox(
    engine: &Engine,
    mailbox: MailboxId,
    key: IdempotencyKey,
) -> Result<Response, MiraError> {
    let account = account_of_mailbox(engine, &mailbox)?;
    let kind = ActionKind::EmptyMailbox { mailbox };
    submit_mailbox_action(engine, account, kind, key)
}
