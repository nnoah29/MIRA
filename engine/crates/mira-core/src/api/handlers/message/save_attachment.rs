//! Enregistrement d'une pièce jointe dans un fichier : opération longue, annulable, atomique.

use std::path::PathBuf;

use mira_vocabulary::error::codes::{invalid_input, not_found};
use mira_vocabulary::{AttachmentId, ErrorCategory, MiraError};

use crate::api::contract::{AttachmentTarget, Response};
use crate::api::engine::Engine;
use crate::fetching::save_attachment;
use crate::operations::run_in_background;

/// Refuse une destination vide
fn require_destination(destination: &str) -> Result<PathBuf, MiraError> {
    if destination.trim().is_empty() {
        return Err(MiraError::new(
            ErrorCategory::InvalidInput,
            invalid_input::MISSING_PARAMETER,
            "la destination de la pièce jointe est vide",
        ));
    }
    Ok(PathBuf::from(destination))
}

/// Refuse une pièce jointe inconnue avant de démarrer l'opération
fn require_attachment(engine: &Engine, id: &AttachmentId) -> Result<(), MiraError> {
    let known = engine.database.read(|tx| tx.get_attachment(id))?;
    known.map(|_| ()).ok_or_else(|| {
        MiraError::new(
            ErrorCategory::NotFound,
            not_found::ATTACHMENT,
            "pièce jointe inconnue",
        )
    })
}

/// Démarre l'enregistrement en arrière-plan et rend la fiche de l'opération
pub(crate) fn handle_save_attachment(
    engine: &Engine,
    target: AttachmentTarget,
) -> Result<Response, MiraError> {
    let AttachmentTarget {
        attachment,
        destination,
    } = target;
    let target = require_destination(&destination)?;
    require_attachment(engine, &attachment)?;
    let handle = engine.operations.start("save_attachment");
    let info = engine.operations.get(handle.id())?;
    let task = engine.fetch_task(handle.clone());
    run_in_background(handle, async move {
        save_attachment(&task, &attachment, &target).await
    });
    Ok(Response::Operation(info))
}
