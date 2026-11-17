//! Permissions après extraction et recherche de l'exécutable : le moteur fixe les modes, pas l'archive.

use std::fs;
use std::path::{Path, PathBuf};

use mira_vocabulary::MiraError;

use super::extract_check::corrupted;
use super::file_mode::{apply_mode, is_executable};

/// Remplace les modes de l'archive : dossiers 0755, fichiers 0755 s'ils étaient exécutables, 0644 sinon.
pub(super) fn normalize_modes(root: &Path) -> Result<(), MiraError> {
    for item in fs::read_dir(root).map_err(|_| corrupted())? {
        let path = item.map_err(|_| corrupted())?.path();
        let meta = fs::symlink_metadata(&path).map_err(|_| corrupted())?;
        if meta.is_dir() {
            apply_mode(&path, 0o755).map_err(|_| corrupted())?;
            normalize_modes(&path)?;
        } else if meta.is_file() {
            let executable = is_executable(&meta);
            let mode = if executable { 0o755 } else { 0o644 };
            apply_mode(&path, mode).map_err(|_| corrupted())?;
        }
    }
    Ok(())
}

/// Cherche un fichier de ce nom sous `root` (profondeur 4 au plus) et le rend exécutable.
pub(crate) fn find_executable(root: &Path, name: &str) -> Option<PathBuf> {
    find_in(root, name, 4).inspect(|path| {
        let _ = apply_mode(path, 0o755);
    })
}

/// Parcours borné en profondeur, sans suivre les liens symboliques.
fn find_in(dir: &Path, name: &str, depth: u32) -> Option<PathBuf> {
    for item in fs::read_dir(dir).ok()?.flatten() {
        let path = item.path();
        let meta = fs::symlink_metadata(&path).ok()?;
        if meta.is_file() && item.file_name() == name {
            return Some(path);
        }
        if meta.is_dir()
            && depth > 0
            && let Some(found) = find_in(&path, name, depth - 1)
        {
            return Some(found);
        }
    }
    None
}
