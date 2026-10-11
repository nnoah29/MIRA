//! Inverse et test « déjà en place » des étapes sur les apparitions.

use mira_vocabulary::{MailboxId, MessageId, MiraError};

use crate::actions::model::LocalStep;
use crate::storage::Tx;

/// Étape qui rétablit l'état actuel de l'apparition : sa présence (UID, supprimé) ou son absence
pub(super) fn snapshot(
    tx: &Tx<'_>,
    message: &MessageId,
    mailbox: &MailboxId,
) -> Result<Vec<LocalStep>, MiraError> {
    let step = match tx.get_appearance(message, mailbox)? {
        Some(current) => LocalStep::PutAppearance {
            message: message.clone(),
            mailbox: mailbox.clone(),
            uid: (current.uid != 0).then_some(current.uid),
            deleted: current.deleted,
        },
        None => LocalStep::DropAppearance {
            message: message.clone(),
            mailbox: mailbox.clone(),
        },
    };
    Ok(vec![step])
}

/// Dit si l'apparition existe déjà avec l'état voulu (un message ou une boîte absents comptent comme en place)
pub(super) fn put_in_effect(
    tx: &Tx<'_>,
    message: &MessageId,
    mailbox: &MailboxId,
    uid: Option<u32>,
    deleted: bool,
) -> Result<bool, MiraError> {
    if tx.get_message(message)?.is_none() || tx.get_mailbox(mailbox)?.is_none() {
        return Ok(true);
    }
    Ok(tx.get_appearance(message, mailbox)?.is_some_and(|current| {
        current.deleted == deleted && uid.is_none_or(|wanted| wanted == current.uid)
    }))
}

/// Dit si l'apparition est déjà absente
pub(super) fn drop_in_effect(
    tx: &Tx<'_>,
    message: &MessageId,
    mailbox: &MailboxId,
) -> Result<bool, MiraError> {
    Ok(tx.get_appearance(message, mailbox)?.is_none())
}

/// Dit si l'apparition a déjà la marque « supprimé » voulue (une apparition absente compte comme en place)
pub(super) fn deleted_in_effect(
    tx: &Tx<'_>,
    message: &MessageId,
    mailbox: &MailboxId,
    deleted: bool,
) -> Result<bool, MiraError> {
    Ok(tx
        .get_appearance(message, mailbox)?
        .is_none_or(|current| current.deleted == deleted))
}
