//! Rapport d'un import de règles, mémorisé avec sa clé d'idempotence.

use mira_vocabulary::{IdempotencyKey, MiraError};
use rusqlite::{OptionalExtension, params};

use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Rapport JSON mémorisé sous cette clé, s'il y en a un
    pub fn rule_import_result(&self, key: &IdempotencyKey) -> Result<Option<String>, MiraError> {
        let found = self
            .sql()
            .query_row(
                "SELECT result FROM rule_idempotency WHERE key = ?1",
                [key.0.as_str()],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()
            .map_err(storage_error)?;
        Ok(found.flatten())
    }

    /// Mémorise le rapport JSON d'une clé déjà enregistrée
    pub fn set_rule_import_result(
        &self,
        key: &IdempotencyKey,
        result: &str,
    ) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "UPDATE rule_idempotency SET result = ?2 WHERE key = ?1",
                params![key.0, result],
            )
            .map_err(storage_error)?;
        Ok(())
    }
}
