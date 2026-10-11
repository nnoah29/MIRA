//! Inverse et test « déjà en place » des étapes sur les boîtes.

use mira_vocabulary::{Mailbox, MailboxId, MiraError};

use crate::actions::model::LocalStep;
use crate::storage::Tx;

/// Inverse d'une suppression : recréer la boîte puis chacune de ses apparitions
pub(super) fn before_drop(tx: &Tx<'_>, id: &MailboxId) -> Result<Vec<LocalStep>, MiraError> {
    let Some(mailbox) = tx.get_mailbox(id)? else {
        return Ok(Vec::new());
    };
    let mut steps = vec![LocalStep::CreateMailbox { mailbox }];
    for appearance in tx.appearances_in_mailbox(id)? {
        steps.push(LocalStep::PutAppearance {
            message: appearance.message,
            mailbox: id.clone(),
            uid: (appearance.uid != 0).then_some(appearance.uid),
            deleted: appearance.deleted,
        });
    }
    Ok(steps)
}

/// Inverse d'un renommage : reprendre le nom actuel
pub(super) fn before_rename(tx: &Tx<'_>, id: &MailboxId) -> Result<Vec<LocalStep>, MiraError> {
    Ok(tx
        .get_mailbox(id)?
        .map(|mailbox| LocalStep::RenameMailbox {
            mailbox: id.clone(),
            name: mailbox.name,
            display_name: mailbox.display_name,
        })
        .into_iter()
        .collect())
}

/// Dit si une boîte de ce nom existe déjà
pub(super) fn create_in_effect(tx: &Tx<'_>, mailbox: &Mailbox) -> Result<bool, MiraError> {
    Ok(tx
        .find_mailbox_by_name(&mailbox.account, &mailbox.name)?
        .is_some())
}

/// Dit si la boîte est déjà absente
pub(super) fn drop_in_effect(tx: &Tx<'_>, id: &MailboxId) -> Result<bool, MiraError> {
    Ok(tx.get_mailbox(id)?.is_none())
}

/// Dit si la boîte porte déjà le nom voulu (une boîte absente compte comme en place)
pub(super) fn rename_in_effect(tx: &Tx<'_>, id: &MailboxId, name: &str) -> Result<bool, MiraError> {
    Ok(tx
        .get_mailbox(id)?
        .is_none_or(|mailbox| mailbox.name == name))
}
