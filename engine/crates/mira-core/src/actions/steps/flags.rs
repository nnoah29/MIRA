//! Étapes sur les drapeaux du message (D-14) : un seul jeu de drapeaux par message.

use mira_vocabulary::{Flags, MailboxId, MessageId, MiraError};

use crate::storage::Tx;

/// Fixe « lu » sur le message ; rend les boîtes dont les compteurs changent
pub(super) fn set_seen(
    tx: &Tx<'_>,
    message: &MessageId,
    seen: bool,
) -> Result<Vec<MailboxId>, MiraError> {
    edit_flags(tx, message, |flags| flags.seen = seen)
}

/// Fixe « suivi » sur le message ; rend les boîtes concernées
pub(super) fn set_flagged(
    tx: &Tx<'_>,
    message: &MessageId,
    flagged: bool,
) -> Result<Vec<MailboxId>, MiraError> {
    edit_flags(tx, message, |flags| flags.flagged = flagged)
}

/// Applique une modification aux drapeaux d'un message connu ; un message absent est ignoré
fn edit_flags(
    tx: &Tx<'_>,
    message: &MessageId,
    change: impl FnOnce(&mut Flags),
) -> Result<Vec<MailboxId>, MiraError> {
    let Some(summary) = tx.get_message(message)? else {
        return Ok(Vec::new());
    };
    let mut flags = summary.flags;
    change(&mut flags);
    tx.update_message_flags(message, flags)?;
    Ok(summary.mailboxes)
}
