//! Étapes d'ajout et de retrait d'une apparition (libellés).

use mira_vocabulary::{MailboxId, MessageId};

use crate::actions::model::LocalStep;

/// Ajoute chaque message à la boîte (libellé supplémentaire)
pub(super) fn add_to(targets: &[MessageId], mailbox: &MailboxId) -> Vec<LocalStep> {
    targets
        .iter()
        .map(|message| put_step(message, mailbox))
        .collect()
}

/// Étape de retrait d'une apparition
pub(super) fn drop_step(message: &MessageId, mailbox: &MailboxId) -> LocalStep {
    LocalStep::DropAppearance {
        message: message.clone(),
        mailbox: mailbox.clone(),
    }
}

/// Étape d'ajout provisoire d'une apparition
pub(super) fn put_step(message: &MessageId, mailbox: &MailboxId) -> LocalStep {
    LocalStep::PutAppearance {
        message: message.clone(),
        mailbox: mailbox.clone(),
        uid: None,
        deleted: false,
    }
}
