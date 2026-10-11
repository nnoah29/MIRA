//! Étapes des actions sur les boîtes : créer, renommer, supprimer, vider.

use mira_vocabulary::{Mailbox, MailboxId, MiraError};

use crate::actions::model::LocalStep;
use crate::storage::Tx;

/// Création locale de la boîte
pub(super) fn create(mailbox: &Mailbox) -> Vec<LocalStep> {
    vec![LocalStep::CreateMailbox {
        mailbox: mailbox.clone(),
    }]
}

/// Renommage local de la boîte
pub(super) fn rename(mailbox: &MailboxId, name: &str, display_name: &str) -> Vec<LocalStep> {
    vec![LocalStep::RenameMailbox {
        mailbox: mailbox.clone(),
        name: name.to_string(),
        display_name: display_name.to_string(),
    }]
}

/// Suppression locale de la boîte
pub(super) fn delete(mailbox: &MailboxId) -> Vec<LocalStep> {
    vec![LocalStep::DropMailbox {
        mailbox: mailbox.clone(),
    }]
}

/// Marque « supprimé » toutes les apparitions encore visibles de la boîte
pub(super) fn empty(tx: &Tx<'_>, mailbox: &MailboxId) -> Result<Vec<LocalStep>, MiraError> {
    Ok(tx
        .appearances_in_mailbox(mailbox)?
        .into_iter()
        .filter(|appearance| !appearance.deleted)
        .map(|appearance| LocalStep::SetAppearanceDeleted {
            message: appearance.message,
            mailbox: mailbox.clone(),
            deleted: true,
        })
        .collect())
}
