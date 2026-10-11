//! Résolution des actions sur les boîtes : créer, renommer, supprimer, vider.

use mira_vocabulary::{AccountId, Mailbox, MailboxId, MailboxRole, MiraError};

use super::checks::require_mailbox;
use super::naming::{child_name, delimiter_of, renamed_name};
use crate::actions::errors::incompatible::system_mailbox;
use crate::actions::errors::input::invalid_parameter;
use crate::actions::model::LocalIntent;
use crate::ports::IdGenerator;
use crate::storage::Tx;

/// Créer une boîte, éventuellement sous un parent ; le nom ne doit pas déjà exister
pub(super) fn create(
    tx: &Tx<'_>,
    account: &AccountId,
    name: &str,
    parent: Option<&MailboxId>,
    ids: &dyn IdGenerator,
) -> Result<LocalIntent, MiraError> {
    let leaf = name.trim();
    if leaf.is_empty() {
        return Err(invalid_parameter("le nom de la boîte est vide"));
    }
    let parent = parent
        .map(|id| require_mailbox(tx, account, id))
        .transpose()?;
    let full_name = child_name(parent.as_ref(), leaf);
    require_free_name(tx, account, &full_name)?;
    let id = MailboxId::from_raw(ids.next_id());
    Ok(LocalIntent::CreateMailbox {
        mailbox: new_mailbox(id, account, full_name, leaf, parent.as_ref()),
    })
}

/// Boîte neuve de rôle « autre », vide, sans repère de synchronisation
fn new_mailbox(
    id: MailboxId,
    account: &AccountId,
    name: String,
    leaf: &str,
    parent: Option<&Mailbox>,
) -> Mailbox {
    Mailbox {
        id,
        account: account.clone(),
        name,
        display_name: leaf.to_string(),
        delimiter: Some(parent.map_or_else(|| "/".to_string(), delimiter_of)),
        parent: parent.map(|mailbox| mailbox.id.clone()),
        role: MailboxRole::Other,
        sync_enabled: true,
        marker: None,
        total: 0,
        unread: 0,
    }
}

/// Renommer une boîte non système ; le nouveau nom ne doit pas déjà exister
pub(super) fn rename(
    tx: &Tx<'_>,
    account: &AccountId,
    id: &MailboxId,
    name: &str,
) -> Result<LocalIntent, MiraError> {
    let mailbox = require_mailbox(tx, account, id)?;
    let leaf = name.trim();
    if leaf.is_empty() {
        return Err(invalid_parameter("le nom de la boîte est vide"));
    }
    if mailbox.role.is_system() {
        return Err(system_mailbox(id));
    }
    let full_name = renamed_name(&mailbox, leaf);
    require_free_name(tx, account, &full_name)?;
    Ok(LocalIntent::RenameMailbox {
        mailbox: id.clone(),
        name: full_name,
        display_name: leaf.to_string(),
    })
}

/// Supprimer une boîte non système
pub(super) fn delete(
    tx: &Tx<'_>,
    account: &AccountId,
    id: &MailboxId,
) -> Result<LocalIntent, MiraError> {
    let mailbox = require_mailbox(tx, account, id)?;
    if mailbox.role.is_system() {
        return Err(system_mailbox(id));
    }
    Ok(LocalIntent::DeleteMailbox {
        mailbox: id.clone(),
    })
}

/// Vider une boîte (messages supprimés définitivement)
pub(super) fn empty(
    tx: &Tx<'_>,
    account: &AccountId,
    id: &MailboxId,
) -> Result<LocalIntent, MiraError> {
    require_mailbox(tx, account, id)?;
    Ok(LocalIntent::EmptyMailbox {
        mailbox: id.clone(),
    })
}

/// Refuse un nom déjà pris par une autre boîte du compte
fn require_free_name(tx: &Tx<'_>, account: &AccountId, name: &str) -> Result<(), MiraError> {
    if tx.find_mailbox_by_name(account, name)?.is_some() {
        return Err(invalid_parameter("une boîte de ce nom existe déjà"));
    }
    Ok(())
}
