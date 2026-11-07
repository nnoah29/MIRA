//! Préparation du dossier et du fichier de socket : permissions privées, socket périmée remplacée.

use std::fs::File;
use std::os::unix::fs::{DirBuilderExt, FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt};
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
        Err(error) if error.kind() == std::io::ErrorKind::ConnectionRefused => {
            std::fs::remove_file(path)
        }
        Err(error) => Err(error),
    }
}

/// Prend le verrou exclusif `<socket>.lock` ; rend `AddrInUse` si un autre démon démarre ou tourne déjà sur cette socket.
pub(super) fn lock_socket(path: &Path) -> std::io::Result<File> {
    let mut name = path.as_os_str().to_owned();
    name.push(".lock");
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .mode(0o600)
        .open(Path::new(&name))?;
    match lock.try_lock() {
        Ok(()) => Ok(lock),
        Err(std::fs::TryLockError::WouldBlock) => Err(std::io::ErrorKind::AddrInUse.into()),
        Err(std::fs::TryLockError::Error(error)) => Err(error),
    }
}

/// Refuse un dossier qui n'appartient pas à l'utilisateur du démon, même en 0700.
pub fn check_directory_owner(directory: &Path, own_uid: u32) -> std::io::Result<()> {
    if std::fs::metadata(directory)?.uid() == own_uid {
        return Ok(());
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "le dossier de la socket appartient à un autre utilisateur",
    ))
}
