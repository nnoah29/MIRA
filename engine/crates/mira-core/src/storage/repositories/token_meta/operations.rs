//! Écriture, lecture et suppression de l'expiration d'un jeton d'accès.

use mira_vocabulary::{AccountId, MiraError, Timestamp};
use rusqlite::{OptionalExtension, params};

use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Enregistre l'expiration du jeton d'accès d'un compte ; `None` efface l'information
    pub fn set_token_expiry(
        &self,
        account: &AccountId,
        expires_at: Option<Timestamp>,
    ) -> Result<(), MiraError> {
        let Some(instant) = expires_at else {
            return self.delete_token_expiry(account);
        };
        self.sql()
            .execute(
                "INSERT INTO account_token_meta (account_id, expires_at) VALUES (?1, ?2) ON CONFLICT(account_id) DO UPDATE SET expires_at = excluded.expires_at",
                params![account.as_str(), instant.as_millis()],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Relit l'expiration connue du jeton d'accès d'un compte
    pub fn token_expiry(&self, account: &AccountId) -> Result<Option<Timestamp>, MiraError> {
        let millis: Option<i64> = self
            .sql()
            .query_row(
                "SELECT expires_at FROM account_token_meta WHERE account_id = ?1",
                [account.as_str()],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        Ok(millis.map(Timestamp::from_millis))
    }

    /// Oublie l'expiration du jeton d'accès d'un compte
    pub fn delete_token_expiry(&self, account: &AccountId) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "DELETE FROM account_token_meta WHERE account_id = ?1",
                [account.as_str()],
            )
            .map_err(storage_error)?;
        Ok(())
    }
}
