//! Fichier de clé du trousseau chiffré : 32 octets, lisible par le seul propriétaire.

use std::path::Path;

use mira_assembly::mira_vocabulary::error::codes::invalid_input;
use mira_assembly::mira_vocabulary::{ErrorCategory, MiraError};

/// Erreur fixe : le contenu ou le chemin de la clé n'apparaît jamais.
fn invalid() -> MiraError {
    MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::PARAMETER,
        "fichier de clé du trousseau invalide",
    )
}

/// Lit la clé de 32 octets ; refuse un fichier ouvert au groupe ou aux autres, ou de mauvaise taille.
pub fn read_secrets_key(path: &Path) -> Result<Vec<u8>, MiraError> {
    let metadata = std::fs::metadata(path).map_err(|_| invalid())?;
    #[cfg(unix)]
    if std::os::unix::fs::PermissionsExt::mode(&metadata.permissions()) & 0o077 != 0 {
        return Err(invalid());
    }
    let key = std::fs::read(path).map_err(|_| invalid())?;
    if metadata.len() != 32 || key.len() != 32 {
        return Err(invalid());
    }
    Ok(key)
}
