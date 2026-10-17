//! Conversion des erreurs SQLite en erreurs du contrat (catégorie « stockage local »).

use mira_vocabulary::error::codes::local_storage;
use mira_vocabulary::{ErrorCategory, MiraError};

/// Convertit une erreur SQLite en erreur du contrat, sans exposer de donnée
pub(crate) fn storage_error(error: rusqlite::Error) -> MiraError {
    let code = match error.sqlite_error_code() {
        Some(rusqlite::ErrorCode::DiskFull) => local_storage::DISK_FULL,
        Some(rusqlite::ErrorCode::DatabaseCorrupt) | Some(rusqlite::ErrorCode::NotADatabase) => {
            local_storage::CORRUPTED
        }
        _ => local_storage::IO,
    };
    MiraError::new(
        ErrorCategory::LocalStorage,
        code,
        format!("base locale : {error}"),
    )
}
