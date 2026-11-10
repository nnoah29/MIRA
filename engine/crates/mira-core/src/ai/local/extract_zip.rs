//! Extraction d'une archive `.zip` dans un dossier de préparation, avec les garanties de l'extraction `.tar.gz`.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use mira_vocabulary::MiraError;
use zip::ZipArchive;

use super::extract::with_staging;
use super::extract_check::corrupted;
use super::zip_names::check_zip_name;

/// Extrait l'archive dans `dest` ; toute entrée piégée, tout dépassement de `max_bytes` annule tout.
pub fn extract_zip(archive: &Path, dest: &Path, max_bytes: u64) -> Result<(), MiraError> {
    with_staging(dest, |stage| unpack_zip(archive, stage, max_bytes))
}

/// Contrôle puis écrit chaque entrée ; le total réellement écrit est borné, quoi que l'archive annonce.
fn unpack_zip(archive: &Path, stage: &Path, max_bytes: u64) -> Result<(), MiraError> {
    let file = File::open(archive).map_err(|_| corrupted())?;
    let mut zip = ZipArchive::new(file).map_err(|_| corrupted())?;
    let mut remaining = max_bytes;
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index).map_err(|_| corrupted())?;
        let name = entry.name().map_err(|_| corrupted())?.into_owned();
        check_zip_name(&name)?;
        if entry.is_symlink() {
            return Err(corrupted());
        }
        let target = stage.join(&name);
        if entry.is_dir() {
            std::fs::create_dir_all(&target).map_err(|_| corrupted())?;
        } else {
            remaining = write_entry(&mut entry, &target, remaining)?;
        }
    }
    Ok(())
}

/// Écrit un fichier en lisant au plus `remaining` octets ; en rend ce qu'il reste du plafond.
fn write_entry(entry: &mut impl Read, target: &Path, remaining: u64) -> Result<u64, MiraError> {
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|_| corrupted())?;
    }
    let mut out = File::create(target).map_err(|_| corrupted())?;
    let written = std::io::copy(&mut entry.take(remaining.saturating_add(1)), &mut out)
        .map_err(|_| corrupted())?;
    if written > remaining {
        return Err(corrupted());
    }
    Ok(remaining - written)
}
