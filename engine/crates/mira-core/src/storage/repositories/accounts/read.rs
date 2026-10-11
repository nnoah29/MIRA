//! Lectures sur les comptes.

use mira_vocabulary::{Account, AccountId, MiraError};
use rusqlite::OptionalExtension;

use super::rows::{COLUMNS, RawAccount, account_not_found};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Relit un compte, ou rien s'il n'existe pas
    pub fn get_account(&self, id: &AccountId) -> Result<Option<Account>, MiraError> {
        let raw = self
            .sql()
            .query_row(
                &format!("SELECT {COLUMNS} FROM accounts WHERE id = ?1"),
                [id.as_str()],
                RawAccount::from_row,
            )
            .optional()
            .map_err(storage_error)?;
        raw.map(RawAccount::into_account).transpose()
    }

    /// Liste tous les comptes, du plus ancien au plus récent
    pub fn list_accounts(&self) -> Result<Vec<Account>, MiraError> {
        let mut statement = self
            .sql()
            .prepare(&format!(
                "SELECT {COLUMNS} FROM accounts ORDER BY created_at, id"
            ))
            .map_err(storage_error)?;
        let raws = statement
            .query_map([], RawAccount::from_row)
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        raws.into_iter().map(RawAccount::into_account).collect()
    }

    /// Dit si un compte est marqué « en suppression »
    pub fn account_is_deleting(&self, id: &AccountId) -> Result<bool, MiraError> {
        let flag = self
            .sql()
            .query_row(
                "SELECT deleting FROM accounts WHERE id = ?1",
                [id.as_str()],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(storage_error)?;
        flag.map(|value| value != 0)
            .ok_or_else(|| account_not_found(id))
    }
}
