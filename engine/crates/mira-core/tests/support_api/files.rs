//! Recherche d'une valeur dans tous les fichiers d'un dossier (base SQLite, journal WAL compris).

use std::fs;
use std::path::{Path, PathBuf};

/// Liste récursivement tous les fichiers sous `root`
pub fn files_under(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            found.extend(files_under(&path));
        } else {
            found.push(path);
        }
    }
    found
}

/// Chemins des fichiers sous `root` dont le contenu brut contient `needle`
pub fn files_containing(root: &Path, needle: &str) -> Vec<PathBuf> {
    files_under(root)
        .into_iter()
        .filter(|path| {
            let bytes = fs::read(path).unwrap_or_default();
            bytes.windows(needle.len()).any(|w| w == needle.as_bytes())
        })
        .collect()
}
