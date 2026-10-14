//! Fin d'une resynchronisation : retrait des apparitions non confirmées, sauf les apparitions locales.

use mira_vocabulary::{Mailbox, MessageId, MiraError};

use crate::storage::Tx;

/// Apparitions provisoires (déplacement ou libellé hors ligne) : message et état « supprimé ».
fn provisional_in(tx: &Tx<'_>, mailbox: &Mailbox) -> Result<Vec<(MessageId, bool)>, MiraError> {
    let known = tx.appearances_in_mailbox(&mailbox.id)?;
    Ok(known
        .into_iter()
        .filter(|a| a.needs_revalidation && a.uid == 0)
        .map(|a| (a.message, a.deleted))
        .collect())
}

/// Supprime les apparitions non revalidées ; les apparitions provisoires survivent (actions en attente).
pub fn purge_unconfirmed(tx: &Tx<'_>, mailbox: &Mailbox) -> Result<(), MiraError> {
    let provisional = provisional_in(tx, mailbox)?;
    tx.purge_unrevalidated(&mailbox.id)?;
    for (message, deleted) in &provisional {
        tx.add_local_appearance(message, &mailbox.id, *deleted)?;
    }
    Ok(())
}
