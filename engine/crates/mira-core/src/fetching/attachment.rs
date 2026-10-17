//! Enregistrement d'une pièce jointe dans un fichier : cache local d'abord, téléchargement si besoin (12.7).

use std::path::Path;

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{Attachment, AttachmentId, ErrorCategory, EventKind, MiraError};

use super::atomic_write::write_atomically;
use super::body::source_for_message;
use super::task::FetchTask;
use crate::mime::attachment_bytes;

/// Erreur : pièce jointe inconnue ou contenu illisible
fn attachment_missing(message: &str) -> MiraError {
    MiraError::new(ErrorCategory::NotFound, not_found::ATTACHMENT, message)
}

/// Relit la pièce jointe et la position de sa partie, ou `not_found.attachment`
fn load_attachment(task: &FetchTask, id: &AttachmentId) -> Result<(Attachment, u32), MiraError> {
    task.database.read(|tx| {
        let attachment = tx
            .get_attachment(id)?
            .ok_or_else(|| attachment_missing("pièce jointe inconnue"))?;
        let part = tx
            .attachment_part_index(id)?
            .ok_or_else(|| attachment_missing("pièce jointe inconnue"))?;
        Ok((attachment, part))
    })
}

/// Contenu de la pièce jointe : le cache s'il existe, sinon extrait de la source (téléchargée au besoin)
async fn attachment_content(
    task: &FetchTask,
    attachment: &Attachment,
    part: u32,
) -> Result<Option<Vec<u8>>, MiraError> {
    let cached = task
        .database
        .read(|tx| tx.attachment_cache_path(&attachment.id))?;
    if let Some(bytes) = cached.and_then(|path| std::fs::read(path).ok()) {
        return Ok(Some(bytes));
    }
    let Some(raw) = source_for_message(task, &attachment.message).await? else {
        return Ok(None);
    };
    attachment_bytes(&raw, part)
        .map(Some)
        .ok_or_else(|| attachment_missing("contenu de la pièce jointe illisible"))
}

/// Range le contenu dans le cache des pièces jointes et note son chemin ; `false` si annulé
async fn store_in_cache(
    task: &FetchTask,
    attachment: &Attachment,
    bytes: &[u8],
) -> Result<bool, MiraError> {
    let path = task.cache_directory.join(attachment.id.as_str());
    if !write_atomically(&path, bytes, &task.handle).await? {
        return Ok(false);
    }
    let text = path.to_string_lossy().to_string();
    task.database
        .write(|tx| tx.set_attachment_cache_path(&attachment.id, Some(&text)))?;
    Ok(true)
}

/// Annonce par événement que la pièce jointe est enregistrée
fn announce_ready(task: &FetchTask, attachment: &Attachment) -> Result<(), MiraError> {
    let account = task.database.read(|tx| {
        Ok(tx
            .get_message(&attachment.message)?
            .map(|message| message.account))
    })?;
    if let Some(account) = account {
        task.events.emit(EventKind::AttachmentReady {
            account,
            attachment: attachment.id.clone(),
            operation: task.handle.id().clone(),
        })?;
    }
    Ok(())
}

/// Écrit la pièce jointe à `destination` de façon atomique ; annulée, elle ne laisse aucun fichier
pub async fn save_attachment(
    task: &FetchTask,
    id: &AttachmentId,
    destination: &Path,
) -> Result<(), MiraError> {
    let (attachment, part) = load_attachment(task, id)?;
    let Some(bytes) = attachment_content(task, &attachment, part).await? else {
        return Ok(());
    };
    if attachment.cache == mira_vocabulary::CacheState::NotDownloaded
        && !store_in_cache(task, &attachment, &bytes).await?
    {
        return Ok(());
    }
    if !write_atomically(destination, &bytes, &task.handle).await? {
        return Ok(());
    }
    announce_ready(task, &attachment)
}
