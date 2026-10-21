//! Fichiers de cache des pièces jointes d'un compte : suppression avec le compte ou avec son miroir.

use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path};

use mira_vocabulary::{AccountId, MiraError};

use crate::storage::Database;

/// Vrai si le fichier est sous le dossier de cache, sans remontée par `..`
pub(crate) fn is_inside_cache(file: &Path, cache_directory: &Path) -> bool {
    file.starts_with(cache_directory)
        && !file
            .components()
            .any(|component| component == Component::ParentDir)
}

/// Supprime les fichiers de cache des pièces jointes du compte, ceux du dossier de cache seulement
pub(crate) fn remove_account_cache_files(
    database: &Database,
    account: &AccountId,
    cache_directory: &Path,
) -> Result<(), MiraError> {
    let paths = database.read(|tx| tx.attachment_cache_paths_of_account(account))?;
    for path in paths {
        let file = Path::new(&path);
        if !is_inside_cache(file, cache_directory) {
            continue;
        }
        if let Err(error) = fs::remove_file(file)
            && error.kind() != ErrorKind::NotFound
        {
            tracing::warn!(kind = ?error.kind(), "fichier de cache non supprimé");
        }
    }
    Ok(())
}

/// Jette le miroir du compte (D-18), ses traces et les fichiers de cache de ses pièces jointes ; configuration et intentions restent
pub fn discard_mirror_and_cache(
    database: &Database,
    account: &AccountId,
    cache_directory: &Path,
) -> Result<(), MiraError> {
    remove_account_cache_files(database, account, cache_directory)?;
    database.discard_mirror_of(account)
}
