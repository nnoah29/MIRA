//! Préparation du dossier et du fichier de socket : permissions privées, socket périmée remplacée.

use std::os::unix::fs::{DirBuilderExt, FileTypeExt, PermissionsExt};
use std::path::Path;

/// Crée le dossier en 0700, ou refuse un dossier existant ouvert au groupe ou aux autres.
pub(super) fn prepare_directory(directory: &Path) -> std::io::Result<()> {
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(directory)?;
    if std::fs::metadata(directory)?.permissions().mode() & 0o077 != 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "le dossier de la socket doit être privé",
        ));
    }
    Ok(())
}

/// Supprime une socket périmée ; refuse si quelqu'un écoute ou si ce n'est pas une socket.
pub(super) fn clear_stale(path: &Path) -> std::io::Result<()> {
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return Ok(());
    };
    if !metadata.file_type().is_socket() {
        return Err(std::io::ErrorKind::AlreadyExists.into());
    }
    match std::os::unix::net::UnixStream::connect(path) {
        Ok(_) => Err(std::io::ErrorKind::AddrInUse.into()),
        Err(_) => std::fs::remove_file(path),
    }
}
