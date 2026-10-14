//! Purge du cache des pièces jointes : fichiers du cache supprimés, chemins oubliés en base.

use std::fs;
use std::path::Path;

use crate::storage::Database;
use mira_vocabulary::MiraError;

/// Supprime les fichiers du cache connus de la base et oublie leurs chemins ; rend le nombre de fichiers retirés
pub fn purge_attachment_cache(
    database: &Database,
    cache_directory: &Path,
) -> Result<u64, MiraError> {
    let paths = database.write(|tx| tx.clear_attachment_cache_paths())?;
    let mut removed = 0;
    for path in paths {
        let file = Path::new(&path);
        if file.starts_with(cache_directory) && fs::remove_file(file).is_ok() {
            removed += 1;
        }
    }
    Ok(removed)
}
