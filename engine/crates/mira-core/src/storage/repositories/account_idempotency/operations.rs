//! Enregistrement et relecture des clés d'idempotence de l'ajout de compte.

use mira_vocabulary::{AccountId, IdempotencyKey, MiraError};
use rusqlite::{OptionalExtension, params};

use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Retrouve le compte créé avec cette clé, avec l'adresse de la commande d'origine
    pub fn account_for_idempotency_key(
        &self,
        key: &IdempotencyKey,
    ) -> Result<Option<(AccountId, String)>, MiraError> {
        self.sql()
            .query_row(
                "SELECT account_id, address FROM account_idempotency WHERE key = ?1",
                [key.0.as_str()],
                |row| Ok((AccountId::from_raw(row.get::<_, String>(0)?), row.get(1)?)),
            )
            .optional()
            .map_err(storage_error)
    }

    /// Mémorise que cette clé a produit ce compte
    pub fn record_account_idempotency(
        &self,
        key: &IdempotencyKey,
        address: &str,
        account: &AccountId,
    ) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "INSERT INTO account_idempotency (key, address, account_id) VALUES (?1, ?2, ?3)",
                params![key.0, address, account.as_str()],
            )
            .map_err(storage_error)?;
        Ok(())
    }
}
