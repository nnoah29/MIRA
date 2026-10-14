//! Écriture atomique d'un fichier : temporaire voisin, copie par blocs annulable, puis renommage (12.7).

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use mira_vocabulary::{ErrorCategory, MiraError};

use crate::operations::OperationHandle;
use crate::profile::errors::io_error;

const CHUNK_BYTES: usize = 256 * 1024;

/// Chemin du fichier temporaire voisin de la destination, propre à l'opération
fn temporary_path(destination: &Path, handle: &OperationHandle) -> Result<PathBuf, MiraError> {
    let name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| invalid_destination("la destination doit désigner un fichier"))?;
    let temporary = format!(".{name}.{}.mira-part", handle.id().as_str());
    Ok(destination.with_file_name(temporary))
}

/// Erreur de destination invalide
fn invalid_destination(message: &str) -> MiraError {
    MiraError::new(
        ErrorCategory::InvalidInput,
        mira_vocabulary::error::codes::invalid_input::PARAMETER,
        message,
    )
}

/// Copie les octets par blocs dans le temporaire ; `false` si l'annulation est demandée en route
async fn copy_chunks(
    file: &mut File,
    bytes: &[u8],
    handle: &OperationHandle,
) -> Result<bool, MiraError> {
    let total = bytes.len() as u64;
    let mut written = 0u64;
    for chunk in bytes.chunks(CHUNK_BYTES) {
        if handle.is_cancelled() {
            return Ok(false);
        }
        file.write_all(chunk)
            .map_err(|error| io_error("écriture du fichier", &error))?;
        written += chunk.len() as u64;
        handle.progress(written, total);
        tokio::task::yield_now().await;
    }
    Ok(!handle.is_cancelled())
}

/// Écrit `bytes` dans `destination` sans jamais y exposer un fichier partiel ; `false` si annulé
pub async fn write_atomically(
    destination: &Path,
    bytes: &[u8],
    handle: &OperationHandle,
) -> Result<bool, MiraError> {
    let temporary = temporary_path(destination, handle)?;
    let mut file =
        File::create_new(&temporary).map_err(|error| io_error("création du fichier", &error))?;
    let outcome = copy_chunks(&mut file, bytes, handle).await;
    let finished = outcome.and_then(|complete| {
        file.sync_all()
            .map_err(|error| io_error("écriture du fichier", &error))?;
        Ok(complete)
    });
    match finished {
        Ok(true) => fs::rename(&temporary, destination)
            .map(|()| true)
            .map_err(|error| discard(&temporary, io_error("renommage du fichier", &error))),
        Ok(false) => {
            let _ = fs::remove_file(&temporary);
            Ok(false)
        }
        Err(error) => Err(discard(&temporary, error)),
    }
}

/// Supprime le temporaire et rend l'erreur d'origine
fn discard(temporary: &Path, error: MiraError) -> MiraError {
    let _ = fs::remove_file(temporary);
    error
}
