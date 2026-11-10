//! Téléchargement vérifié d'un fichier épinglé : reprise, taille et SHA-256 contrôlés avant installation.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use mira_vocabulary::error::codes::{incompatible_state, local_storage};
use mira_vocabulary::{ErrorCategory, MiraError};

use super::download_transfer::{fetch_into, part_size};
use super::download_verify::verify_and_install;
use crate::auth::loopback::require_secure_url;
use crate::ports::SystemResources;

/// Drapeau levé par l'appelant pour interrompre le transfert.
pub type CancelFlag = Arc<AtomicBool>;

/// Ce qu'il faut pour télécharger un fichier épinglé.
#[derive(Debug)]
pub struct DownloadJob<'a> {
    pub url: &'a str,
    pub sha256: &'a str,
    pub size: u64,
    pub dest: &'a Path,
    pub resources: &'a dyn SystemResources,
}

/// Télécharge vers `<dest>.part` en reprenant si possible, vérifie taille et somme, puis installe sous `dest`.
pub async fn download_file(
    job: &DownloadJob<'_>,
    progress: &(dyn Fn(u64, u64) + Send + Sync),
    cancel: &CancelFlag,
) -> Result<(), MiraError> {
    require_secure_url(job.url)?;
    if already_installed(job.dest, job.size).await {
        return Ok(());
    }
    let part = part_path(job.dest);
    prepare_folder(job.dest).await?;
    let existing = part_size(&part, job.size).await;
    check_space(job, existing)?;
    if existing < job.size {
        fetch_into(job, &part, existing, progress, cancel).await?;
    }
    verify_and_install(job, &part).await
}

/// Dit si le fichier final est déjà là à la bonne taille (sa somme a été vérifiée à l'installation).
async fn already_installed(dest: &Path, size: u64) -> bool {
    tokio::fs::metadata(dest)
        .await
        .is_ok_and(|m| m.is_file() && m.len() == size)
}

/// Chemin du fichier partiel : celui du fichier final avec `.part` ajouté.
pub(crate) fn part_path(dest: &Path) -> PathBuf {
    let mut name = dest.as_os_str().to_os_string();
    name.push(".part");
    PathBuf::from(name)
}

/// Crée le dossier de destination.
async fn prepare_folder(dest: &Path) -> Result<(), MiraError> {
    let folder = dest.parent().unwrap_or_else(|| Path::new("."));
    tokio::fs::create_dir_all(folder)
        .await
        .map_err(|_| io_error())
}

/// Refuse d'aller plus loin si l'espace libre ne couvre pas le reste à télécharger plus 10 %.
fn check_space(job: &DownloadJob<'_>, existing: u64) -> Result<(), MiraError> {
    let folder = job.dest.parent().unwrap_or_else(|| Path::new("."));
    let needed = job
        .size
        .saturating_sub(existing)
        .saturating_add(job.size / 10);
    match job.resources.free_disk_bytes(folder) {
        Some(free) if free < needed => Err(MiraError::new(
            ErrorCategory::IncompatibleState,
            incompatible_state::AI_LOCAL_DISK_SPACE,
            "espace disque insuffisant pour le modèle local",
        )),
        _ => Ok(()),
    }
}

/// Erreur d'écriture ou de lecture locale, sans chemin ni détail.
pub(crate) fn io_error() -> MiraError {
    MiraError::new(
        ErrorCategory::LocalStorage,
        local_storage::IO,
        "erreur d'accès au dossier du modèle local",
    )
}
