//! Extraction d'une archive `.tar.gz` dans un dossier de préparation, puis renommage : jamais à moitié rempli.

use std::fs::File;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use mira_vocabulary::MiraError;
use tar::Archive;

use super::extract_check::{check_entry, corrupted};
use super::extract_modes::normalize_modes;

/// Extrait l'archive dans `dest` ; toute entrée piégée, tout dépassement de `max_bytes` annule tout.
pub fn extract_tar_gz(archive: &Path, dest: &Path, max_bytes: u64) -> Result<(), MiraError> {
    with_staging(dest, |stage| unpack_all(archive, stage, max_bytes))
}

/// Remplit un dossier de préparation, normalise les modes, puis le renomme en `dest` ; un échec n'en laisse rien.
pub(super) fn with_staging(
    dest: &Path,
    fill: impl FnOnce(&Path) -> Result<(), MiraError>,
) -> Result<(), MiraError> {
    let stage = staging_of(dest);
    let _ = std::fs::remove_dir_all(&stage);
    std::fs::create_dir_all(&stage).map_err(|_| corrupted())?;
    let outcome = fill(&stage).and_then(|()| normalize_modes(&stage));
    if let Err(error) = outcome {
        let _ = std::fs::remove_dir_all(&stage);
        return Err(error);
    }
    let _ = std::fs::remove_dir_all(dest);
    std::fs::rename(&stage, dest).map_err(|_| corrupted())
}

/// Dossier de préparation voisin de la destination.
fn staging_of(dest: &Path) -> PathBuf {
    let mut name = dest.as_os_str().to_os_string();
    name.push(".staging");
    PathBuf::from(name)
}

/// Contrôle puis écrit chaque entrée dans le dossier de préparation.
fn unpack_all(archive: &Path, stage: &Path, max_bytes: u64) -> Result<(), MiraError> {
    let file = File::open(archive).map_err(|_| corrupted())?;
    let mut tar = Archive::new(GzDecoder::new(file));
    let mut total = 0u64;
    for entry in tar.entries().map_err(|_| corrupted())? {
        let mut entry = entry.map_err(|_| corrupted())?;
        check_entry(&entry, &mut total, max_bytes)?;
        if !entry.unpack_in(stage).map_err(|_| corrupted())? {
            return Err(corrupted());
        }
    }
    Ok(())
}
