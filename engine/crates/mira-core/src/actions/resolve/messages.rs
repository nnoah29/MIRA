//! Résolution des actions sur des messages vers une intention locale (D-17 pour les libellés).

use mira_vocabulary::{
    AccountId, ArchiveStrategy, MailboxId, MailboxModel, MailboxRole, MiraError, ProviderQuirks,
};

use super::checks::{require_mailbox, require_role_mailbox};
use crate::actions::errors::incompatible::multiple_labels_unsupported;
use crate::actions::model::LocalIntent;
use crate::storage::Tx;

/// Déplacer : le message ne reste que dans la boîte cible
pub(super) fn move_to(
    tx: &Tx<'_>,
    account: &AccountId,
    to: &MailboxId,
) -> Result<LocalIntent, MiraError> {
    require_mailbox(tx, account, to)?;
    Ok(LocalIntent::MoveTo {
        mailbox: to.clone(),
    })
}

/// Ajouter un libellé : refusé sur un compte à dossiers (D-17)
pub(super) fn add_to(
    tx: &Tx<'_>,
    account: &AccountId,
    mailbox: &MailboxId,
    quirks: &ProviderQuirks,
) -> Result<LocalIntent, MiraError> {
    if quirks.mailbox_model == MailboxModel::Folders {
        return Err(multiple_labels_unsupported());
    }
    require_mailbox(tx, account, mailbox)?;
    Ok(LocalIntent::AddTo {
        mailbox: mailbox.clone(),
    })
}

/// Retirer d'une boîte : un libellé sur un compte à libellés, un archivage sur un compte à dossiers
pub(super) fn remove_from(
    tx: &Tx<'_>,
    account: &AccountId,
    mailbox: &MailboxId,
    quirks: &ProviderQuirks,
) -> Result<LocalIntent, MiraError> {
    require_mailbox(tx, account, mailbox)?;
    match quirks.mailbox_model {
        MailboxModel::Labels => Ok(LocalIntent::DropFrom {
            mailbox: mailbox.clone(),
        }),
        MailboxModel::Folders => archive(tx, account, quirks),
    }
}

/// Archiver : retirer de la réception ou déplacer vers la boîte d'archive, selon le service
pub(super) fn archive(
    tx: &Tx<'_>,
    account: &AccountId,
    quirks: &ProviderQuirks,
) -> Result<LocalIntent, MiraError> {
    match quirks.archive {
        ArchiveStrategy::RemoveFromInbox => Ok(LocalIntent::DropFrom {
            mailbox: require_role_mailbox(tx, account, MailboxRole::Inbox)?,
        }),
        ArchiveStrategy::MoveToArchiveMailbox => move_to_role(tx, account, MailboxRole::Archive),
    }
}

/// Déplacer vers la boîte portant le rôle donné (corbeille, spam, réception…)
pub(super) fn move_to_role(
    tx: &Tx<'_>,
    account: &AccountId,
    role: MailboxRole,
) -> Result<LocalIntent, MiraError> {
    Ok(LocalIntent::MoveTo {
        mailbox: require_role_mailbox(tx, account, role)?,
    })
}
