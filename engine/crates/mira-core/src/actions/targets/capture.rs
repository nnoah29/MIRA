//! Relevé des cibles serveur d'une action avant que son effet local ne modifie le miroir.

use mira_vocabulary::{AccountId, ActionKind, MailboxId, MessageId, MiraError};

use super::model::{AppearanceTarget, MailboxTarget, MessageTarget, ServerTargets};
use crate::storage::Tx;

/// Relève messages (identifiant stable, apparitions) et boîtes nommées par l'action
pub(crate) fn capture_server_targets(
    tx: &Tx<'_>,
    account: &AccountId,
    kind: &ActionKind,
    targets: &[MessageId],
) -> Result<ServerTargets, MiraError> {
    let mut captured = ServerTargets::default();
    for message in targets {
        captured
            .messages
            .push(message_target(tx, account, message)?);
    }
    for mailbox in mailboxes_named_by(kind) {
        if let Some(target) = mailbox_target(tx, &mailbox)? {
            captured.mailboxes.push(target);
        }
    }
    Ok(captured)
}

/// Boîtes que le type d'action désigne explicitement
fn mailboxes_named_by(kind: &ActionKind) -> Vec<MailboxId> {
    match kind {
        ActionKind::Move { to } => vec![to.clone()],
        ActionKind::AddToMailbox { mailbox }
        | ActionKind::RemoveFromMailbox { mailbox }
        | ActionKind::RenameMailbox { mailbox, .. }
        | ActionKind::DeleteMailbox { mailbox }
        | ActionKind::EmptyMailbox { mailbox } => vec![mailbox.clone()],
        ActionKind::CreateMailbox { parent, .. } => parent.iter().cloned().collect(),
        ActionKind::MarkRead { .. }
        | ActionKind::MarkFlagged { .. }
        | ActionKind::Archive
        | ActionKind::Trash
        | ActionKind::DeletePermanently
        | ActionKind::MarkSpam { .. }
        | ActionKind::SaveDraft { .. }
        | ActionKind::Send { .. } => Vec::new(),
    }
}

/// Cible d'une boîte : son nom et sa validité d'UID actuels
fn mailbox_target(tx: &Tx<'_>, id: &MailboxId) -> Result<Option<MailboxTarget>, MiraError> {
    Ok(tx.get_mailbox(id)?.map(|mailbox| MailboxTarget {
        mailbox: mailbox.id,
        name: mailbox.name,
        uid_validity: mailbox.marker.map(|marker| marker.uid_validity),
    }))
}

/// Cible d'un message : identifiant stable et apparitions non supprimées
fn message_target(
    tx: &Tx<'_>,
    account: &AccountId,
    message: &MessageId,
) -> Result<MessageTarget, MiraError> {
    let mut appearances = Vec::new();
    for appearance in tx.appearances_of_message(message)? {
        if appearance.deleted {
            continue;
        }
        let Some(mailbox) = tx.get_mailbox(&appearance.mailbox)? else {
            continue;
        };
        appearances.push(AppearanceTarget {
            mailbox: mailbox.id,
            mailbox_name: mailbox.name,
            uid_validity: mailbox.marker.map(|marker| marker.uid_validity),
            uid: appearance.uid,
        });
    }
    Ok(MessageTarget {
        message: message.clone(),
        stable_id: tx.stable_server_id_of(account, message)?,
        appearances,
    })
}
