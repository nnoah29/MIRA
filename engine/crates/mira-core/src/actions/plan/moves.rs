//! Étapes des actions qui déplacent, retirent ou suppriment des messages.

use mira_vocabulary::{Appearance, MailboxId, MessageId, MiraError};

use super::labels::{drop_step, put_step};
use crate::actions::model::LocalStep;
use crate::storage::Tx;

/// Range chaque message dans la seule boîte cible : retrait des autres apparitions visibles, ajout dans la cible
pub(super) fn move_to(
    tx: &Tx<'_>,
    targets: &[MessageId],
    to: &MailboxId,
) -> Result<Vec<LocalStep>, MiraError> {
    let mut steps = Vec::new();
    for message in targets {
        for appearance in visible_appearances(tx, message)? {
            if &appearance.mailbox != to {
                steps.push(drop_step(message, &appearance.mailbox));
            }
        }
        steps.push(put_step(message, to));
    }
    Ok(steps)
}

/// Retire chaque message de la boîte, s'il y apparaît
pub(super) fn drop_from(
    tx: &Tx<'_>,
    targets: &[MessageId],
    mailbox: &MailboxId,
) -> Result<Vec<LocalStep>, MiraError> {
    let mut steps = Vec::new();
    for message in targets {
        if tx.get_appearance(message, mailbox)?.is_some() {
            steps.push(drop_step(message, mailbox));
        }
    }
    Ok(steps)
}

/// Marque « supprimé » chaque apparition visible de chaque message
pub(super) fn mark_deleted(
    tx: &Tx<'_>,
    targets: &[MessageId],
) -> Result<Vec<LocalStep>, MiraError> {
    let mut steps = Vec::new();
    for message in targets {
        for appearance in visible_appearances(tx, message)? {
            steps.push(LocalStep::SetAppearanceDeleted {
                message: message.clone(),
                mailbox: appearance.mailbox,
                deleted: true,
            });
        }
    }
    Ok(steps)
}

/// Apparitions du message qui ne sont pas marquées « supprimé »
fn visible_appearances(tx: &Tx<'_>, message: &MessageId) -> Result<Vec<Appearance>, MiraError> {
    Ok(tx
        .appearances_of_message(message)?
        .into_iter()
        .filter(|appearance| !appearance.deleted)
        .collect())
}
