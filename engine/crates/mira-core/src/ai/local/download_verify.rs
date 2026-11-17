//! Vérification finale d'un fichier téléchargé : taille, SHA-256, puis renommage atomique.

use std::path::Path;

use mira_vocabulary::error::codes::server;
use mira_vocabulary::{ErrorCategory, MiraError};
use sha2::{Digest, Sha256};
use tokio::io::AsyncReadExt;

use super::download::{DownloadJob, io_error};

/// Erreur : le fichier reçu ne correspond pas à l'empreinte épinglée.
pub(super) fn mismatch() -> MiraError {
    MiraError::new(
        ErrorCategory::Server,
        server::AI_LOCAL_CHECKSUM_MISMATCH,
        "le fichier téléchargé ne correspond pas à la somme de contrôle attendue",
    )
}

/// Calcule le SHA-256 du fichier, lu par blocs d'un Mio.
async fn sha256_of(path: &Path) -> Result<String, MiraError> {
    let mut file = tokio::fs::File::open(path).await.map_err(|_| io_error())?;
    let mut hasher = Sha256::new();
    let mut block = vec![0u8; 1 << 20];
    loop {
        let read = file.read(&mut block).await.map_err(|_| io_error())?;
        if read == 0 {
            break;
        }
        hasher.update(&block[..read]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

/// Contrôle taille et somme du `.part` ; en cas d'écart le supprime, sinon l'installe sous le nom final.
pub(super) async fn verify_and_install(
    job: &DownloadJob<'_>,
    part: &Path,
) -> Result<(), MiraError> {
    let size = tokio::fs::metadata(part)
        .await
        .map_err(|_| io_error())?
        .len();
    let hash = if size == job.size {
        sha256_of(part).await?
    } else {
        String::new()
    };
    if size != job.size || !hash.eq_ignore_ascii_case(job.sha256) {
        let _ = tokio::fs::remove_file(part).await;
        return Err(mismatch());
    }
    tokio::fs::rename(part, job.dest)
        .await
        .map_err(|_| io_error())
}
