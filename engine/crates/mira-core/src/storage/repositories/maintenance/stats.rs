//! Statistiques de volume du miroir local.

use mira_vocabulary::MiraError;
use serde::{Deserialize, Serialize};

use crate::storage::Tx;
use crate::storage::errors::storage_error;

/// Volumes du miroir : nombres de lignes et octets de sources brutes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageStats {
    pub mailboxes: u64,
    pub messages: u64,
    pub appearances: u64,
    pub threads: u64,
    pub raw_sources: u64,
    pub raw_bytes: u64,
    pub compressed_bytes: u64,
    pub attachments: u64,
    pub attachments_cached: u64,
}

impl Tx<'_> {
    /// Mesure les volumes du miroir (tous comptes confondus)
    pub fn storage_stats(&self) -> Result<StorageStats, MiraError> {
        Ok(StorageStats {
            mailboxes: self.scalar("SELECT COUNT(*) FROM mailboxes")?,
            messages: self.scalar("SELECT COUNT(*) FROM messages")?,
            appearances: self.scalar("SELECT COUNT(*) FROM appearances")?,
            threads: self.scalar("SELECT COUNT(*) FROM threads")?,
            raw_sources: self.scalar("SELECT COUNT(*) FROM raw_sources")?,
            raw_bytes: self.scalar("SELECT COALESCE(SUM(raw_size), 0) FROM raw_sources")?,
            compressed_bytes: self
                .scalar("SELECT COALESCE(SUM(LENGTH(compressed)), 0) FROM raw_sources")?,
            attachments: self.scalar("SELECT COUNT(*) FROM attachments")?,
            attachments_cached: self
                .scalar("SELECT COUNT(*) FROM attachments WHERE cache_path IS NOT NULL")?,
        })
    }

    /// Lit un entier positif rendu par une requête sans paramètre
    fn scalar(&self, sql: &str) -> Result<u64, MiraError> {
        let value: i64 = self
            .sql()
            .query_row(sql, [], |row| row.get(0))
            .map_err(storage_error)?;
        Ok(u64::try_from(value).unwrap_or(0))
    }
}
