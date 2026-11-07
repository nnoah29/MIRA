//! Écriture du jeton : aléa, fichier temporaire privé et dossier privé.

use std::io::Write;
use std::path::{Path, PathBuf};

use ring::rand::{SecureRandom, SystemRandom};

use super::file::TOKEN_BYTES;

/// 256 bits tirés du générateur du système, en hexadécimal.
pub(super) fn random_hex() -> std::io::Result<String> {
    let mut bytes = [0u8; TOKEN_BYTES];
    SystemRandom::new()
        .fill(&mut bytes)
        .map_err(|_| std::io::Error::other("aléa indisponible"))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

/// Crée le dossier du jeton et ses parents manquants, en n'autorisant que le propriétaire (0700 sous Unix).
pub(super) fn create_parent(parent: &Path) -> std::io::Result<()> {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder.create(parent)
}

/// Chemin d'un fichier temporaire à côté du jeton, au nom imprévisible.
pub(super) fn temp_beside(path: &Path) -> std::io::Result<PathBuf> {
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned());
    let suffix = &random_hex()?[..16];
    Ok(path.with_file_name(format!(".{}.{suffix}.tmp", name.unwrap_or_default())))
}

/// Écrit le jeton en entier dans un fichier neuf privé (0600 sous Unix).
pub(super) fn write_private(path: &Path, token: &str) -> std::io::Result<()> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    options.open(path)?.write_all(token.as_bytes())
}
