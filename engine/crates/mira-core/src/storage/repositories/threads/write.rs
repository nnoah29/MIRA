//! Écritures sur les fils : création idempotente par clé de racine, effacement.

use mira_vocabulary::{AccountId, MiraError, ThreadId};
use rusqlite::params;

use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Rend le fil de (compte, clé de racine), en le créant sous `candidate` s'il n'existe pas
    pub fn upsert_thread(
        &self,
        account: &AccountId,
        root_key: &str,
        subject: &str,
        candidate: &ThreadId,
    ) -> Result<ThreadId, MiraError> {
        self.sql()
            .execute(
                "INSERT INTO threads (id, account_id, subject, root_key) VALUES (?1, ?2, ?3, ?4) ON CONFLICT (account_id, root_key) DO NOTHING",
                params![candidate.as_str(), account.as_str(), subject, root_key],
            )
            .map_err(storage_error)?;
        self.sql()
            .query_row(
                "SELECT id FROM threads WHERE account_id = ?1 AND root_key = ?2",
                params![account.as_str(), root_key],
                |row| row.get::<_, String>(0),
            )
            .map(ThreadId::from_raw)
            .map_err(storage_error)
    }

    /// Efface tous les fils d'un compte et détache ses messages (table dérivée)
    pub fn clear_threads(&self, account: &AccountId) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "UPDATE messages SET thread_id = NULL WHERE account_id = ?1",
                [account.as_str()],
            )
            .map_err(storage_error)?;
        self.sql()
            .execute(
                "DELETE FROM threads WHERE account_id = ?1",
                [account.as_str()],
            )
            .map_err(storage_error)?;
        Ok(())
    }
}
