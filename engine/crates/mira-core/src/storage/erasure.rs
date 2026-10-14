//! Effacement des traces : fusion de l'index de recherche et vidage du journal WAL après une suppression.

use mira_vocabulary::{AccountId, MiraError};

use super::database::Database;
use super::errors::storage_error;

const MERGE_SEARCH_INDEX: &str = "INSERT INTO messages_fts(messages_fts) VALUES('optimize')";
const TRUNCATE_JOURNAL: &str = "PRAGMA wal_checkpoint(TRUNCATE)";

impl Database {
    /// Fusionne l'index de recherche puis reporte et tronque le journal, pour que les données supprimées ne survivent nulle part
    pub fn erase_traces(&self) -> Result<(), MiraError> {
        let connection = self.lock();
        connection
            .execute(MERGE_SEARCH_INDEX, [])
            .map_err(storage_error)?;
        let blocked: i64 = connection
            .query_row(TRUNCATE_JOURNAL, [], |row| row.get(0))
            .map_err(storage_error)?;
        if blocked != 0 {
            tracing::warn!("journal non tronqué : un lecteur est encore actif");
        }
        Ok(())
    }

    /// Jette le miroir d'un compte puis efface ses traces dans l'index et le journal
    pub fn discard_mirror_of(&self, account: &AccountId) -> Result<(), MiraError> {
        self.write(|tx| tx.discard_account_mirror(account))?;
        self.erase_traces()
    }
}
