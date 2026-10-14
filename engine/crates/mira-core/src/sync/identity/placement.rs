//! Placement d'un message dans une boîte : nouvelle apparition, revalidation ou changement d'UID.

use mira_vocabulary::{Appearance, MailboxId, MessageId, MiraError};

use crate::storage::Tx;

/// Fait correspondre l'apparition locale au serveur ; rend `true` si la présence ou l'UID a changé.
pub fn place_appearance(
    tx: &Tx<'_>,
    message: &MessageId,
    mailbox: &MailboxId,
    uid: u32,
    deleted: bool,
) -> Result<bool, MiraError> {
    let fresh = Appearance {
        message: message.clone(),
        mailbox: mailbox.clone(),
        uid,
        deleted,
        needs_revalidation: false,
    };
    match tx.get_appearance(message, mailbox)? {
        None => tx.add_appearance(&fresh).map(|()| true),
        Some(known) if known.needs_revalidation => {
            tx.revalidate_appearance(message, mailbox, uid)?;
            tx.set_appearance_deleted(message, mailbox, deleted)?;
            Ok(true)
        }
        Some(known) if known.uid == uid => {
            tx.set_appearance_deleted(message, mailbox, deleted)?;
            Ok(false)
        }
        Some(_) => replace_appearance(tx, &fresh),
    }
}

/// Remplace une apparition dont l'UID a changé côté serveur.
fn replace_appearance(tx: &Tx<'_>, fresh: &Appearance) -> Result<bool, MiraError> {
    tx.remove_appearance(&fresh.message, &fresh.mailbox)?;
    tx.add_appearance(fresh).map(|()| true)
}
