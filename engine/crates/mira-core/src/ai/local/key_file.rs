//! Clé du serveur local : tirée au hasard à chaque lancement, écrite dans un fichier 0600, jamais en argument.

use std::path::Path;

use mira_vocabulary::MiraError;
use rand::{Rng, SeedableRng, rngs::StdRng};
use tokio::io::AsyncWriteExt;

use super::download::io_error;
use super::file_mode::apply_mode;

/// Tire 32 octets au hasard et les rend en hexadécimal (64 caractères).
pub(super) fn new_key() -> String {
    let bytes: [u8; 32] = StdRng::from_os_rng().random();
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Écrit la clé dans `<run_dir>/api-key` (dossier 0700, fichier 0600), en remplaçant la précédente.
pub(super) async fn write_key_file(
    run_dir: &Path,
    key: &str,
) -> Result<std::path::PathBuf, MiraError> {
    tokio::fs::create_dir_all(run_dir)
        .await
        .map_err(|_| io_error())?;
    set_mode(run_dir, 0o700).await?;
    let path = run_dir.join("api-key");
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)
        .await
        .map_err(|_| io_error())?;
    set_mode(&path, 0o600).await?;
    file.write_all(key.as_bytes())
        .await
        .map_err(|_| io_error())?;
    Ok(path)
}

/// Fixe les permissions d'un chemin.
async fn set_mode(path: &Path, mode: u32) -> Result<(), MiraError> {
    let owned = path.to_path_buf();
    tokio::task::spawn_blocking(move || apply_mode(&owned, mode))
        .await
        .map_err(|_| io_error())?
        .map_err(|_| io_error())
}
