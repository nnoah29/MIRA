//! Compteurs de boîte (total, non lus) : recalculés depuis les apparitions après chaque modification.

use mira_vocabulary::{AccountId, Mailbox, MiraError};

use crate::storage::{Database, Tx};

/// Recalcule les compteurs de toutes les boîtes d'un compte, dans la transaction de l'appelant
pub fn refresh_account_counters(tx: &Tx<'_>, account: &AccountId) -> Result<(), MiraError> {
    for mailbox in tx.list_mailboxes(account)? {
        tx.refresh_mailbox_counters(&mailbox.id)?;
    }
    Ok(())
}

/// Rend les boîtes d'un compte avec des compteurs cohérents avec les messages actuels
pub fn mailboxes_with_counters(
    db: &Database,
    account: &AccountId,
) -> Result<Vec<Mailbox>, MiraError> {
    db.write(|tx| {
        refresh_account_counters(tx, account)?;
        tx.list_mailboxes(account)
    })
}
