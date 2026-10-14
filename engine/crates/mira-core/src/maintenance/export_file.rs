//! Écriture d'un fichier d'export : temporaire aléatoire créé en exclusif puis renommage, droits 0600.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{ErrorCategory, MiraError};

use crate::profile::errors::io_error;

/// Chemin du temporaire voisin de la destination, au suffixe aléatoire
fn temporary_path(destination: &Path) -> Result<PathBuf, MiraError> {
    let name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            MiraError::new(
                ErrorCategory::InvalidInput,
                invalid_input::PARAMETER,
                "la destination doit désigner un fichier",
            )
        })?;
    let suffix = rand::random::<u64>();
    Ok(destination.with_file_name(format!(".{name}.{suffix:016x}.mira-part")))
}

/// Écrit le contenu de façon atomique ; en cas d'échec, aucun fichier partiel ne reste
pub fn write_export_file(destination: &Path, bytes: &[u8]) -> Result<(), MiraError> {
    let temporary = temporary_path(destination)?;
    let written = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)
        .and_then(|mut file| file.write_all(bytes).and_then(|()| file.sync_all()))
        .and_then(|()| fs::rename(&temporary, destination));
    written.map_err(|error| {
        let _ = fs::remove_file(&temporary);
        io_error("écriture de l'export", &error)
    })
}
