//! Choix de l'extraction selon le format de l'archive : `.zip` ou `.tar.gz`.

use std::path::Path;

use mira_vocabulary::MiraError;

use super::extract::extract_tar_gz;
use super::extract_zip::extract_zip;

/// Suffixe du fichier téléchargé, déduit de l'adresse (sans requête ni ancre).
pub(super) fn archive_suffix(url: &str) -> &'static str {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    if path.to_ascii_lowercase().ends_with(".zip") {
        ".zip"
    } else {
        ".tar.gz"
    }
}

/// Extrait l'archive selon son suffixe.
pub(super) fn extract_archive(
    archive: &Path,
    dest: &Path,
    max_bytes: u64,
) -> Result<(), MiraError> {
    if archive
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("zip"))
    {
        extract_zip(archive, dest, max_bytes)
    } else {
        extract_tar_gz(archive, dest, max_bytes)
    }
}
