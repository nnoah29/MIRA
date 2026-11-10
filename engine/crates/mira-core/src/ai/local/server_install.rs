//! Installation du serveur : archive épinglée téléchargée, vérifiée, extraite, exécutable trouvé.

use std::path::{Path, PathBuf};

use mira_vocabulary::error::codes::incompatible_state;
use mira_vocabulary::{AiLocalServerBuild, ErrorCategory, MiraError};

use super::download::{CancelFlag, DownloadJob, download_file};
use super::extract::extract_tar_gz;
use super::extract_check::corrupted;
use super::extract_modes::find_executable;
use crate::ports::SystemResources;

const MAX_UNPACKED_BYTES: u64 = 500 * 1024 * 1024;

/// Ce qu'il faut pour installer le serveur d'une plateforme dans `root`.
#[derive(Debug)]
pub struct ServerInstall<'a> {
    pub build: &'a AiLocalServerBuild,
    pub root: &'a Path,
    pub resources: &'a dyn SystemResources,
}

/// Nom de la plateforme courante dans le descripteur, ou `None` si elle n'est pas prise en charge.
pub fn current_platform() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => Some("linux-x86_64"),
        ("linux", "aarch64") => Some("linux-aarch64"),
        _ => None,
    }
}

/// Choisit l'archive de la plateforme, sinon `incompatible_state.ai_local_unsupported_platform`.
pub fn select_build<'a>(
    builds: &'a [AiLocalServerBuild],
    platform: &str,
) -> Result<&'a AiLocalServerBuild, MiraError> {
    builds
        .iter()
        .find(|b| b.platform == platform)
        .ok_or_else(|| {
            MiraError::new(
                ErrorCategory::IncompatibleState,
                incompatible_state::AI_LOCAL_UNSUPPORTED_PLATFORM,
                "le modèle local n'est pas pris en charge sur cette plateforme",
            )
        })
}

/// Installe le serveur s'il ne l'est pas déjà et rend le chemin de son exécutable.
pub async fn install_server(
    job: &ServerInstall<'_>,
    progress: &(dyn Fn(u64, u64) + Send + Sync),
    cancel: &CancelFlag,
) -> Result<PathBuf, MiraError> {
    let name = &job.build.executable;
    if let Some(found) = find_executable(job.root, name) {
        return Ok(found);
    }
    let mut archive = job.root.as_os_str().to_os_string();
    archive.push(".tar.gz");
    let archive = PathBuf::from(archive);
    let download = DownloadJob {
        url: &job.build.url,
        sha256: &job.build.sha256,
        size: job.build.size,
        dest: &archive,
        resources: job.resources,
    };
    download_file(&download, progress, cancel).await?;
    let (from, to) = (archive.clone(), job.root.to_path_buf());
    let done = tokio::task::spawn_blocking(move || extract_tar_gz(&from, &to, MAX_UNPACKED_BYTES))
        .await
        .map_err(|_| corrupted())?;
    let _ = tokio::fs::remove_file(&archive).await;
    done?;
    find_executable(job.root, name).ok_or_else(corrupted)
}
