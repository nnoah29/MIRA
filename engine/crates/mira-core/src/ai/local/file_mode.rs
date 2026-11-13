//! Modes de fichiers : appliqués sous Unix, sans effet ailleurs (le serveur local n'y est pas proposé).

use std::fs::Metadata;
use std::path::Path;

/// Fixe le mode d'un chemin.
#[cfg(unix)]
pub(super) fn apply_mode(path: &Path, mode: u32) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
}

/// Sans effet hors Unix.
#[cfg(not(unix))]
pub(super) fn apply_mode(_path: &Path, _mode: u32) -> std::io::Result<()> {
    Ok(())
}

/// Dit si le fichier porte un bit d'exécution.
#[cfg(unix)]
pub(super) fn is_executable(meta: &Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    meta.permissions().mode() & 0o111 != 0
}

/// Hors Unix, aucun bit d'exécution n'existe.
#[cfg(not(unix))]
pub(super) fn is_executable(_meta: &Metadata) -> bool {
    false
}
