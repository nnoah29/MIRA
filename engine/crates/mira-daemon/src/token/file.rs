//! Fichier du jeton : lecture avec contrôle des permissions, création privée.

use std::io::{ErrorKind, Read, Write};
use std::path::Path;

use ring::rand::{SecureRandom, SystemRandom};

const TOKEN_BYTES: usize = 32;

/// Lit le jeton d'un fichier existant ; `None` si le fichier n'existe pas. Refuse un fichier ouvert aux autres.
pub fn read_existing(path: &Path) -> std::io::Result<Option<String>> {
    let mut file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    refuse_shared(&file.metadata()?)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    let text = text.trim().to_string();
    if text.len() == TOKEN_BYTES * 2 && text.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Ok(Some(text));
    }
    Err(std::io::Error::new(
        ErrorKind::InvalidData,
        "jeton illisible",
    ))
}

/// Refuse un fichier lisible ou modifiable par le groupe ou les autres.
#[cfg(unix)]
fn refuse_shared(metadata: &std::fs::Metadata) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    if metadata.permissions().mode() & 0o077 != 0 {
        return Err(std::io::Error::new(
            ErrorKind::PermissionDenied,
            "le fichier du jeton doit être privé",
        ));
    }
    Ok(())
}

/// Sans permissions Unix, rien à contrôler ici (phase Windows).
#[cfg(not(unix))]
fn refuse_shared(_metadata: &std::fs::Metadata) -> std::io::Result<()> {
    Ok(())
}

/// Crée le fichier privé (0600 sous Unix) avec un jeton neuf ; en cas de course, relit celui du gagnant.
pub fn create_private(path: &Path) -> std::io::Result<String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let token = random_hex()?;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    match options.open(path) {
        Ok(mut file) => file.write_all(token.as_bytes()).map(|()| token),
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            read_existing(path)?.ok_or_else(|| ErrorKind::NotFound.into())
        }
        Err(error) => Err(error),
    }
}

/// 256 bits tirés du générateur du système, en hexadécimal.
fn random_hex() -> std::io::Result<String> {
    let mut bytes = [0u8; TOKEN_BYTES];
    SystemRandom::new()
        .fill(&mut bytes)
        .map_err(|_| std::io::Error::other("aléa indisponible"))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
