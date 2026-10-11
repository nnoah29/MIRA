//! Vérifications préalables : cibles du compte, boîtes du compte, boîtes de rôle.

use mira_vocabulary::{AccountId, Mailbox, MailboxId, MailboxRole, MessageId, MiraError};

use crate::actions::errors::input::{invalid_parameter, missing_parameter, role_mailbox_missing};
use crate::storage::Tx;
use crate::storage::repositories::mailboxes::mailbox_not_found;
use crate::storage::repositories::messages::message_not_found;

/// Exige au moins un message, tous connus et tous du compte (sinon `not_found.message`)
pub(super) fn require_targets(
    tx: &Tx<'_>,
    account: &AccountId,
    targets: &[MessageId],
) -> Result<(), MiraError> {
    if targets.is_empty() {
        return Err(missing_parameter("aucun message ciblé"));
    }
    for id in targets {
        match tx.get_message(id)? {
            Some(message) if &message.account == account => {}
            _ => return Err(message_not_found(id)),
        }
    }
    Ok(())
}

/// Refuse des messages ciblés pour une action qui vise une boîte
pub(super) fn require_no_targets(targets: &[MessageId]) -> Result<(), MiraError> {
    if targets.is_empty() {
        return Ok(());
    }
    Err(invalid_parameter(
        "cette action vise une boîte, pas des messages",
    ))
}

/// Exige une boîte connue du compte et la rend (sinon `not_found.mailbox`)
pub(super) fn require_mailbox(
    tx: &Tx<'_>,
    account: &AccountId,
    id: &MailboxId,
) -> Result<Mailbox, MiraError> {
    match tx.get_mailbox(id)? {
        Some(mailbox) if &mailbox.account == account => Ok(mailbox),
        _ => Err(mailbox_not_found(id)),
    }
}

/// Exige la boîte du rôle donné (sinon `server.mailbox_missing`)
pub(super) fn require_role_mailbox(
    tx: &Tx<'_>,
    account: &AccountId,
    role: MailboxRole,
) -> Result<MailboxId, MiraError> {
    tx.find_mailbox_by_role(account, role)?
        .map(|mailbox| mailbox.id)
        .ok_or_else(|| role_mailbox_missing(&format!("{role:?}").to_lowercase()))
}
