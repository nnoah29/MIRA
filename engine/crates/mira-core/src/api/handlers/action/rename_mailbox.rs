//! Renommage d'une boîte ; une boîte système est refusée avant la mise en file.

use mira_vocabulary::{ActionKind, IdempotencyKey, MailboxId, MiraError};

use super::mailbox_submit::{account_of_mailbox, submit_mailbox_action};
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Met en file le renommage de la boîte
pub(crate) fn handle_rename_mailbox(
    engine: &Engine,
    mailbox: MailboxId,
    name: String,
    key: IdempotencyKey,
) -> Result<Response, MiraError> {
    let account = account_of_mailbox(engine, &mailbox)?;
    let kind = ActionKind::RenameMailbox { mailbox, name };
    submit_mailbox_action(engine, account, kind, key)
}
