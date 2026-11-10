//! Contrôle de chaque entrée d'une archive avant son écriture : chemin, lien, type, taille cumulée.

use std::io::Read;
use std::path::{Component, Path};

use mira_vocabulary::error::codes::local_storage;
use mira_vocabulary::{ErrorCategory, MiraError};
use tar::{Entry, EntryType};

/// Erreur d'une archive hostile ou illisible, sans nom d'entrée ni chemin.
pub(super) fn corrupted() -> MiraError {
    MiraError::new(
        ErrorCategory::LocalStorage,
        local_storage::CORRUPTED,
        "archive du serveur local invalide",
    )
}

/// Refuse un chemin absolu ou contenant `..` ; seuls les composants ordinaires et `.` passent.
fn check_relative(path: &Path) -> Result<(), MiraError> {
    let clean = path
        .components()
        .all(|c| matches!(c, Component::Normal(_) | Component::CurDir));
    if clean { Ok(()) } else { Err(corrupted()) }
}

/// Refuse un lien dont la cible, résolue depuis le dossier du lien, sort du dossier d'extraction.
fn check_link_target(entry_path: &Path, target: &Path, hard: bool) -> Result<(), MiraError> {
    let mut depth = if hard {
        0
    } else {
        entry_path.components().count().saturating_sub(1)
    };
    for component in target.components() {
        match component {
            Component::Normal(_) => depth += 1,
            Component::CurDir => {}
            Component::ParentDir if depth > 0 => depth -= 1,
            _ => return Err(corrupted()),
        }
    }
    Ok(())
}

/// Contrôle une entrée : chemin sûr, type admis (fichier, dossier, lien relatif), taille cumulée bornée.
pub(super) fn check_entry<R: Read>(
    entry: &Entry<'_, R>,
    total: &mut u64,
    max_bytes: u64,
) -> Result<(), MiraError> {
    let path = entry.path().map_err(|_| corrupted())?;
    check_relative(&path)?;
    match entry.header().entry_type() {
        EntryType::Regular | EntryType::Directory | EntryType::Continuous => {}
        EntryType::Symlink | EntryType::Link => {
            let hard = entry.header().entry_type() == EntryType::Link;
            let target = entry
                .link_name()
                .map_err(|_| corrupted())?
                .ok_or_else(corrupted)?;
            check_link_target(&path, &target, hard)?;
        }
        _ => return Err(corrupted()),
    }
    *total = total.saturating_add(entry.size());
    if *total > max_bytes {
        Err(corrupted())
    } else {
        Ok(())
    }
}
