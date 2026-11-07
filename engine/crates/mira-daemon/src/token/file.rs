//! Fichier du jeton : lecture avec contrôle des permissions, création privée.

use std::io::{ErrorKind, Read};
use std::path::Path;

use super::publish::{create_parent, random_hex, temp_beside, write_private};

pub(super) const TOKEN_BYTES: usize = 32;

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

/// Crée le fichier privé avec un jeton neuf, publié d'un coup (écrit en entier à côté, puis lié sous son nom) ; en cas de course, relit celui du gagnant.
pub fn create_private(path: &Path) -> std::io::Result<String> {
    if let Some(parent) = path.parent() {
        create_parent(parent)?;
    }
    let token = random_hex()?;
    let temp = temp_beside(path)?;
    write_private(&temp, &token)?;
    let published = std::fs::hard_link(&temp, path);
    let _ = std::fs::remove_file(&temp);
    match published {
        Ok(()) => Ok(token),
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            read_existing(path)?.ok_or_else(|| ErrorKind::NotFound.into())
        }
        Err(error) => Err(error),
    }
}
