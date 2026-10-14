//! Application locale des changements vus sur le serveur : drapeaux (le serveur gagne) et disparitions.

use mira_vocabulary::{MailboxId, MessageId, MiraError};

use crate::imap::FetchedFlags;
use crate::storage::Tx;
use crate::sync::identity::{apply_server_flags, is_deleted, server_flags};

/// Écrase les drapeaux du miroir par ceux du serveur ; rend les messages modifiés (UID inconnus ignorés).
pub fn apply_flag_changes(
    tx: &Tx<'_>,
    mailbox: &MailboxId,
    changes: &[FetchedFlags],
) -> Result<Vec<MessageId>, MiraError> {
    let mut updated = Vec::new();
    for change in changes {
        let Some(appearance) = tx.appearance_by_uid(mailbox, change.uid)? else {
            continue;
        };
        let message = appearance.message;
        let deleted = is_deleted(&change.flags);
        let moved = appearance.deleted != deleted;
        tx.set_appearance_deleted(&message, mailbox, deleted)?;
        let changed = apply_server_flags(tx, &message, server_flags(&change.flags))?;
        if moved || changed {
            updated.push(message);
        }
    }
    Ok(updated)
}

/// Retire les apparitions des UID disparus ; rend les messages touchés (le ménage des orphelins vient après).
pub fn remove_vanished(
    tx: &Tx<'_>,
    mailbox: &MailboxId,
    uids: &[u32],
) -> Result<Vec<MessageId>, MiraError> {
    let mut touched = Vec::new();
    for uid in uids {
        if let Some(appearance) = tx.appearance_by_uid(mailbox, *uid)? {
            tx.remove_appearance(&appearance.message, mailbox)?;
            touched.push(appearance.message);
        }
    }
    Ok(touched)
}
