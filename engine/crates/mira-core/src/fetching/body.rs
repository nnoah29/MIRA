//! Téléchargement explicite du corps d'un message absent du miroir (12.2.1, S6).

use mira_vocabulary::error::codes::incompatible_state;
use mira_vocabulary::{ContentState, ErrorCategory, EventKind, MessageId, MiraError};

use super::download::download_source;
use super::ingest::ingest_source;
use super::locate::{SourceLocation, locate_message};
use super::task::FetchTask;

/// Télécharge la source du message et la range ; ne fait rien si la tâche est annulée avant le rangement
async fn download_and_store(task: &FetchTask, location: &SourceLocation) -> Result<(), MiraError> {
    task.handle.progress(0, 1);
    let raw = download_source(&task.access, location).await?;
    if task.handle.is_cancelled() {
        return Ok(());
    }
    task.database
        .write(|tx| ingest_source(tx, task.ids.as_ref(), &location.message, &raw))?;
    task.events.emit(EventKind::BodyAvailable {
        account: location.account.id.clone(),
        message: location.message.id.clone(),
    })?;
    task.handle.progress(1, 1);
    Ok(())
}

/// Télécharge et range le corps du message ; sans effet s'il est déjà présent ou si la tâche est annulée
pub async fn fetch_message_body(task: &FetchTask, id: &MessageId) -> Result<(), MiraError> {
    let location = task.database.read(|tx| locate_message(tx, id))?;
    if location.message.content == ContentState::BodyPresent || task.handle.is_cancelled() {
        return Ok(());
    }
    download_and_store(task, &location).await
}

/// Rend la source du message, en la téléchargeant d'abord si le miroir ne l'a pas ; `None` si annulé
pub async fn source_for_message(
    task: &FetchTask,
    id: &MessageId,
) -> Result<Option<Vec<u8>>, MiraError> {
    if let Some(raw) = task.database.read(|tx| tx.load_raw_source(id))? {
        return Ok(Some(raw));
    }
    let location = task.database.read(|tx| locate_message(tx, id))?;
    download_and_store(task, &location).await?;
    if task.handle.is_cancelled() {
        return Ok(None);
    }
    task.database
        .read(|tx| tx.load_raw_source(id))?
        .map(Some)
        .ok_or_else(source_not_stored)
}

/// Erreur : la source téléchargée n'a pas pu être relue
fn source_not_stored() -> MiraError {
    MiraError::new(
        ErrorCategory::IncompatibleState,
        incompatible_state::BODY_UNAVAILABLE_OFFLINE,
        "la source du message n'a pas pu être rangée",
    )
}
