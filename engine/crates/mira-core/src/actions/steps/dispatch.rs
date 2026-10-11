//! Routage exhaustif d'une étape vers l'opération qui l'applique, l'inverse ou la reconnaît.

use mira_vocabulary::{MailboxId, MiraError};

use super::{appearances, appearances_inverse, flags, flags_inverse, mailboxes, mailboxes_inverse};
use crate::actions::model::LocalStep;
use crate::storage::Tx;

/// Applique l'étape au miroir ; rend les boîtes dont les compteurs sont à recalculer
pub(crate) fn apply_step(tx: &Tx<'_>, step: &LocalStep) -> Result<Vec<MailboxId>, MiraError> {
    match step {
        LocalStep::SetSeen { message, seen } => flags::set_seen(tx, message, *seen),
        LocalStep::SetFlagged { message, flagged } => flags::set_flagged(tx, message, *flagged),
        LocalStep::PutAppearance {
            message,
            mailbox,
            uid,
            deleted,
        } => appearances::put(tx, message, mailbox, *uid, *deleted),
        LocalStep::DropAppearance { message, mailbox } => {
            appearances::drop_appearance(tx, message, mailbox)
        }
        LocalStep::SetAppearanceDeleted {
            message,
            mailbox,
            deleted,
        } => appearances::set_deleted(tx, message, mailbox, *deleted),
        LocalStep::CreateMailbox { mailbox } => mailboxes::create(tx, mailbox),
        LocalStep::DropMailbox { mailbox } => mailboxes::drop_mailbox(tx, mailbox),
        LocalStep::RenameMailbox {
            mailbox,
            name,
            display_name,
        } => mailboxes::rename(tx, mailbox, name, display_name),
    }
}

/// Lit l'état actuel que l'étape va modifier et rend les étapes qui le rétabliraient
pub(crate) fn capture_inverse(tx: &Tx<'_>, step: &LocalStep) -> Result<Vec<LocalStep>, MiraError> {
    match step {
        LocalStep::SetSeen { message, .. } => flags_inverse::capture_seen(tx, message),
        LocalStep::SetFlagged { message, .. } => flags_inverse::capture_flagged(tx, message),
        LocalStep::PutAppearance {
            message, mailbox, ..
        }
        | LocalStep::DropAppearance { message, mailbox }
        | LocalStep::SetAppearanceDeleted {
            message, mailbox, ..
        } => appearances_inverse::snapshot(tx, message, mailbox),
        LocalStep::CreateMailbox { mailbox } => Ok(vec![LocalStep::DropMailbox {
            mailbox: mailbox.id.clone(),
        }]),
        LocalStep::DropMailbox { mailbox } => mailboxes_inverse::before_drop(tx, mailbox),
        LocalStep::RenameMailbox { mailbox, .. } => mailboxes_inverse::before_rename(tx, mailbox),
    }
}

/// Dit si le miroir reflète déjà l'effet de l'étape (par exemple après une synchro qui l'a rattrapé)
pub(crate) fn step_in_effect(tx: &Tx<'_>, step: &LocalStep) -> Result<bool, MiraError> {
    match step {
        LocalStep::SetSeen { message, seen } => flags_inverse::seen_in_effect(tx, message, *seen),
        LocalStep::SetFlagged { message, flagged } => {
            flags_inverse::flagged_in_effect(tx, message, *flagged)
        }
        LocalStep::PutAppearance {
            message,
            mailbox,
            uid,
            deleted,
        } => appearances_inverse::put_in_effect(tx, message, mailbox, *uid, *deleted),
        LocalStep::DropAppearance { message, mailbox } => {
            appearances_inverse::drop_in_effect(tx, message, mailbox)
        }
        LocalStep::SetAppearanceDeleted {
            message,
            mailbox,
            deleted,
        } => appearances_inverse::deleted_in_effect(tx, message, mailbox, *deleted),
        LocalStep::CreateMailbox { mailbox } => mailboxes_inverse::create_in_effect(tx, mailbox),
        LocalStep::DropMailbox { mailbox } => mailboxes_inverse::drop_in_effect(tx, mailbox),
        LocalStep::RenameMailbox { mailbox, name, .. } => {
            mailboxes_inverse::rename_in_effect(tx, mailbox, name)
        }
    }
}
