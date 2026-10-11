//! Lectures sur les identités.

use mira_vocabulary::{AccountId, Identity, IdentityId, MiraError};
use rusqlite::OptionalExtension;

use super::rows::{COLUMNS, RawIdentity};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Relit une identité, ou rien si elle n'existe pas
    pub fn get_identity(&self, id: &IdentityId) -> Result<Option<Identity>, MiraError> {
        let raw = self
            .sql()
            .query_row(
                &format!("SELECT {COLUMNS} FROM identities WHERE id = ?1"),
                [id.as_str()],
                RawIdentity::from_row,
            )
            .optional()
            .map_err(storage_error)?;
        raw.map(RawIdentity::into_identity).transpose()
    }

    /// Liste les identités d'un compte, l'identité par défaut en premier
    pub fn list_identities(&self, account: &AccountId) -> Result<Vec<Identity>, MiraError> {
        let mut statement = self
            .sql()
            .prepare(&format!(
                "SELECT {COLUMNS} FROM identities WHERE account_id = ?1 ORDER BY is_default DESC, address, id"
            ))
            .map_err(storage_error)?;
        let raws = statement
            .query_map([account.as_str()], RawIdentity::from_row)
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        raws.into_iter().map(RawIdentity::into_identity).collect()
    }
}
