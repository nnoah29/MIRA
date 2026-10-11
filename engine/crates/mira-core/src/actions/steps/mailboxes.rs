//! Étapes sur les boîtes : création, suppression, renommage.

use mira_vocabulary::{Mailbox, MailboxId, MiraError};

use crate::storage::Tx;

/// Crée la boîte localement, sauf si une boîte de ce nom existe déjà
pub(super) fn create(tx: &Tx<'_>, mailbox: &Mailbox) -> Result<Vec<MailboxId>, MiraError> {
    if tx
        .find_mailbox_by_name(&mailbox.account, &mailbox.name)?
        .is_none()
    {
        tx.upsert_mailbox(mailbox)?;
    }
    Ok(Vec::new())
}

/// Supprime la boîte et, en cascade, ses apparitions
pub(super) fn drop_mailbox(tx: &Tx<'_>, mailbox: &MailboxId) -> Result<Vec<MailboxId>, MiraError> {
    if tx.get_mailbox(mailbox)?.is_some() {
        tx.delete_mailbox(mailbox)?;
    }
    Ok(Vec::new())
}

/// Renomme la boîte (et le préfixe de ses descendantes) si elle existe encore
pub(super) fn rename(
    tx: &Tx<'_>,
    mailbox: &MailboxId,
    name: &str,
    display_name: &str,
) -> Result<Vec<MailboxId>, MiraError> {
    if tx.get_mailbox(mailbox)?.is_some() {
        tx.rename_mailbox(mailbox, name, display_name)?;
    }
    Ok(Vec::new())
}
