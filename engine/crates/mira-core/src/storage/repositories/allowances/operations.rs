//! Ajout, test et liste des autorisations de contenu distant.

use mira_vocabulary::error::codes::local_storage;
use mira_vocabulary::{ErrorCategory, MiraError};

use super::model::{RemoteContentAllowance, RemoteContentScope};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Autorise le contenu distant pour une portée et une valeur ; sans effet si déjà autorisé
    pub fn add_remote_content_allowance(
        &self,
        scope: RemoteContentScope,
        value: &str,
    ) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "INSERT OR IGNORE INTO remote_content_allowances (kind, value) VALUES (?1, ?2)",
                [scope.as_column(), scope.normalize(value).as_str()],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Dit si le contenu distant est autorisé pour cette portée et cette valeur
    pub fn is_remote_content_allowed(
        &self,
        scope: RemoteContentScope,
        value: &str,
    ) -> Result<bool, MiraError> {
        let count: i64 = self
            .sql()
            .query_row(
                "SELECT COUNT(*) FROM remote_content_allowances WHERE kind = ?1 AND value = ?2",
                [scope.as_column(), scope.normalize(value).as_str()],
                |row| row.get(0),
            )
            .map_err(storage_error)?;
        Ok(count > 0)
    }

    /// Liste toutes les autorisations, triées par portée puis par valeur
    pub fn list_remote_content_allowances(&self) -> Result<Vec<RemoteContentAllowance>, MiraError> {
        let mut statement = self
            .sql()
            .prepare("SELECT kind, value FROM remote_content_allowances ORDER BY kind, value")
            .map_err(storage_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        rows.into_iter().map(decode_allowance).collect()
    }
}

/// Convertit une ligne (portée, valeur) en autorisation du domaine
fn decode_allowance((kind, value): (String, String)) -> Result<RemoteContentAllowance, MiraError> {
    match RemoteContentScope::from_column(&kind) {
        Some(scope) => Ok(RemoteContentAllowance { scope, value }),
        None => Err(MiraError::new(
            ErrorCategory::LocalStorage,
            local_storage::CORRUPTED,
            "portée d'autorisation inconnue en base",
        )),
    }
}
