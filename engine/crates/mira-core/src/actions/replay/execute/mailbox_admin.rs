//! Création, renommage, suppression et vidage d'une boîte, en vérifiant d'abord l'état du serveur.

use mira_vocabulary::{MiraError, ProviderQuirks};

use super::purge::{effective, purge_selected};
use crate::actions::replay::errors::mailbox_gone;
use crate::imap::ImapClient;

/// Dit si le serveur possède une boîte de ce nom
async fn exists(client: &mut ImapClient, name: &str) -> Result<bool, MiraError> {
    let boxes = client.list_mailboxes().await?;
    Ok(boxes.iter().any(|known| known.name == name))
}

/// Crée la boîte, sauf si elle existe déjà
pub(super) async fn create(client: &mut ImapClient, name: &str) -> Result<(), MiraError> {
    if exists(client, name).await? {
        return Ok(());
    }
    client.create_mailbox(name).await
}

/// Renomme la boîte ; déjà renommée = objectif atteint, introuvable = échec définitif
pub(super) async fn rename(client: &mut ImapClient, from: &str, to: &str) -> Result<(), MiraError> {
    if exists(client, from).await? {
        return client.rename_mailbox(from, to).await;
    }
    if exists(client, to).await? {
        return Ok(());
    }
    Err(mailbox_gone())
}

/// Supprime la boîte, sauf si elle a déjà disparu
pub(super) async fn delete(client: &mut ImapClient, name: &str) -> Result<(), MiraError> {
    if !exists(client, name).await? {
        return Ok(());
    }
    client.delete_mailbox(name).await
}

/// Vide la boîte : tous ses messages sont marqués `\Deleted` puis purgés
pub(super) async fn empty(
    client: &mut ImapClient,
    quirks: &ProviderQuirks,
    name: &str,
) -> Result<(), MiraError> {
    let capabilities = effective(client, quirks);
    client.select(name, None).await?;
    let uids = client.all_uids().await?;
    purge_selected(client, &capabilities, &uids).await
}
