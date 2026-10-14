//! Instant de la dernière synchronisation réussie d'un compte, relu dans le journal des événements.

use mira_vocabulary::{AccountId, MiraError, Timestamp};
use rusqlite::OptionalExtension;

use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Instant du dernier événement `SyncFinished` du compte encore conservé, s'il y en a un
    pub fn last_sync_finished_at(
        &self,
        account: &AccountId,
    ) -> Result<Option<Timestamp>, MiraError> {
        let found: Option<i64> = self
            .sql()
            .query_row(
                "SELECT at FROM events WHERE account_id = ?1 AND category = 'sync' \
                 AND json_extract(kind_json, '$.type') = 'sync_finished' ORDER BY seq DESC LIMIT 1",
                [account.as_str()],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage_error)?;
        Ok(found.map(Timestamp::from_millis))
    }
}
