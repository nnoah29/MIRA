//! Étapes sur les apparitions : ajout (provisoire ou à UID connu), retrait, marque « supprimé ».

use mira_vocabulary::{Appearance, MailboxId, MessageId, MiraError};

use crate::storage::Tx;

/// Garantit l'apparition dans la boîte, avec l'état « supprimé » voulu ; sans UID, elle est provisoire
pub(super) fn put(
    tx: &Tx<'_>,
    message: &MessageId,
    mailbox: &MailboxId,
    uid: Option<u32>,
    deleted: bool,
) -> Result<Vec<MailboxId>, MiraError> {
    if !both_known(tx, message, mailbox)? {
        return Ok(Vec::new());
    }
    match (tx.get_appearance(message, mailbox)?, uid) {
        (None, None) => tx.add_local_appearance(message, mailbox, deleted)?,
        (None, Some(uid)) => place_with_uid(tx, message, mailbox, uid, deleted)?,
        (Some(current), Some(uid)) if current.uid != uid => {
            tx.remove_appearance(message, mailbox)?;
            place_with_uid(tx, message, mailbox, uid, deleted)?;
        }
        (Some(current), _) if current.deleted != deleted => {
            tx.set_appearance_deleted(message, mailbox, deleted)?;
        }
        (Some(_), _) => {}
    }
    Ok(vec![mailbox.clone()])
}

/// Retire l'apparition du message dans la boîte, si elle existe
pub(super) fn drop_appearance(
    tx: &Tx<'_>,
    message: &MessageId,
    mailbox: &MailboxId,
) -> Result<Vec<MailboxId>, MiraError> {
    tx.remove_appearance(message, mailbox)?;
    Ok(vec![mailbox.clone()])
}

/// Marque ou démarque « supprimé » sur l'apparition, si elle existe
pub(super) fn set_deleted(
    tx: &Tx<'_>,
    message: &MessageId,
    mailbox: &MailboxId,
    deleted: bool,
) -> Result<Vec<MailboxId>, MiraError> {
    if tx.get_appearance(message, mailbox)?.is_some() {
        tx.set_appearance_deleted(message, mailbox, deleted)?;
    }
    Ok(vec![mailbox.clone()])
}

/// Dit si le message et la boîte existent encore
fn both_known(tx: &Tx<'_>, message: &MessageId, mailbox: &MailboxId) -> Result<bool, MiraError> {
    Ok(tx.get_message(message)?.is_some() && tx.get_mailbox(mailbox)?.is_some())
}

/// Ajoute l'apparition sous son UID, ou provisoirement si ce UID appartient maintenant à un autre message
fn place_with_uid(
    tx: &Tx<'_>,
    message: &MessageId,
    mailbox: &MailboxId,
    uid: u32,
    deleted: bool,
) -> Result<(), MiraError> {
    let taken = tx
        .appearance_by_uid(mailbox, uid)?
        .is_some_and(|other| &other.message != message);
    if taken {
        return tx.add_local_appearance(message, mailbox, deleted);
    }
    tx.add_appearance(&Appearance {
        message: message.clone(),
        mailbox: mailbox.clone(),
        uid,
        deleted,
        needs_revalidation: false,
    })
}
