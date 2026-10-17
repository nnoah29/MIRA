//! Transformation d'une intention résolue en étapes élémentaires, d'après l'état courant du miroir.

mod flags;
mod labels;
mod mailboxes;
mod moves;

use mira_vocabulary::{MessageId, MiraError};

use super::model::{LocalIntent, LocalStep};
use crate::storage::Tx;

/// Déroule l'intention en étapes sur les messages ciblés ; rejouable à tout moment (rebase)
pub(crate) fn steps_for(
    tx: &Tx<'_>,
    intent: &LocalIntent,
    targets: &[MessageId],
) -> Result<Vec<LocalStep>, MiraError> {
    match intent {
        LocalIntent::SetSeen { seen } => Ok(flags::set_seen(targets, *seen)),
        LocalIntent::SetFlagged { flagged } => Ok(flags::set_flagged(targets, *flagged)),
        LocalIntent::MoveTo { mailbox } => moves::move_to(tx, targets, mailbox),
        LocalIntent::DropFrom { mailbox } => moves::drop_from(tx, targets, mailbox),
        LocalIntent::AddTo { mailbox } => Ok(labels::add_to(targets, mailbox)),
        LocalIntent::MarkDeleted => moves::mark_deleted(tx, targets),
        LocalIntent::CreateMailbox { mailbox } => Ok(mailboxes::create(mailbox)),
        LocalIntent::RenameMailbox {
            mailbox,
            name,
            display_name,
        } => Ok(mailboxes::rename(mailbox, name, display_name)),
        LocalIntent::DeleteMailbox { mailbox } => Ok(mailboxes::delete(mailbox)),
        LocalIntent::EmptyMailbox { mailbox } => mailboxes::empty(tx, mailbox),
        LocalIntent::NoEffect => Ok(Vec::new()),
    }
}
