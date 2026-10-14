//! Suppression d'une boîte ; une boîte système est refusée avant la mise en file.

use mira_vocabulary::{ActionKind, IdempotencyKey, MailboxId, MiraError};

use super::mailbox_submit::{account_of_mailbox, submit_mailbox_action};
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Met en file la suppression de la boîte
pub(crate) fn handle_delete_mailbox(
    engine: &Engine,
    mailbox: MailboxId,
    key: IdempotencyKey,
) -> Result<Response, MiraError> {
    let account = account_of_mailbox(engine, &mailbox)?;
    let kind = ActionKind::DeleteMailbox { mailbox };
    submit_mailbox_action(engine, account, kind, key)
}
