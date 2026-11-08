//! Fichier du port TCP du démon : dit aux clients où se connecter, et sous Windows garantit un seul démon par profil.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Fichier du port, tenu ouvert tant que le démon vit et supprimé à son arrêt.
#[derive(Debug)]
pub struct PortFile {
    file: Option<File>,
    path: PathBuf,
}

/// Ouvre le fichier en écriture ; sous Windows, il n'est partagé qu'en lecture, donc un second démon est refusé.
fn open_exclusive(path: &Path) -> std::io::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(windows)]
    std::os::windows::fs::OpenOptionsExt::share_mode(&mut options, 1);
    options.open(path)
}

/// Crée le dossier du fichier et ses parents manquants, privés au propriétaire (0700 sous Unix).
fn create_parent(path: &Path) -> std::io::Result<()> {
    let Some(parent) = path.parent().filter(|dir| !dir.as_os_str().is_empty()) else {
        return Ok(());
    };
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder.create(parent)
}

/// Traduit le refus de partage de Windows (ERROR_SHARING_VIOLATION, 32) en `AddrInUse` ; toute autre erreur reste telle quelle.
fn held_by_another(error: std::io::Error) -> std::io::Error {
    if error.raw_os_error() == Some(32) && cfg!(windows) {
        return std::io::ErrorKind::AddrInUse.into();
    }
    error
}

impl PortFile {
    /// Écrit le numéro de port ; un fichier périmé est remplacé, un fichier tenu par un autre démon est une erreur `AddrInUse`.
    pub fn publish(path: &Path, port: u16) -> std::io::Result<Self> {
        create_parent(path)?;
        let mut file = open_exclusive(path).map_err(held_by_another)?;
        file.write_all(port.to_string().as_bytes())?;
        file.flush()?;
        Ok(Self {
            file: Some(file),
            path: path.to_path_buf(),
        })
    }
}

impl Drop for PortFile {
    /// Ferme le fichier puis le supprime
    fn drop(&mut self) {
        self.file.take();
        let _ = std::fs::remove_file(&self.path);
    }
}
