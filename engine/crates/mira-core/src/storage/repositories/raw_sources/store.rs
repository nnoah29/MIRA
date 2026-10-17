//! Écriture et lecture de la source brute d'un message.

use mira_vocabulary::{MessageId, MiraError};
use rusqlite::{OptionalExtension, params};

use super::compression::{compress, decompress};
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Enregistre (ou remplace) la source brute d'un message, compressée
    pub fn store_raw_source(&self, message: &MessageId, raw: &[u8]) -> Result<(), MiraError> {
        let compressed = compress(raw)?;
        let size = i64::try_from(raw.len()).unwrap_or(i64::MAX);
        self.sql()
            .execute(
                "INSERT OR REPLACE INTO raw_sources (message_id, compressed, raw_size) VALUES (?1, ?2, ?3)",
                params![message.as_str(), compressed, size],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Relit la source brute d'un message, ou rien si elle n'est pas stockée
    pub fn load_raw_source(&self, message: &MessageId) -> Result<Option<Vec<u8>>, MiraError> {
        let compressed = self
            .sql()
            .query_row(
                "SELECT compressed FROM raw_sources WHERE message_id = ?1",
                [message.as_str()],
                |row| row.get::<_, Vec<u8>>(0),
            )
            .optional()
            .map_err(storage_error)?;
        compressed.map(|bytes| decompress(&bytes)).transpose()
    }
}
