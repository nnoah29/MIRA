//! File transitoire des messages à évaluer (reprise sur crash).

use mira_vocabulary::{MessageId, MiraError};

use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Note un message à évaluer ; sans effet s'il y figure déjà
    pub fn add_pending(&self, message: &MessageId) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "INSERT OR IGNORE INTO rule_pending (message_id) VALUES (?1)",
                [message.as_str()],
            )
            .map_err(storage_error)?;
        Ok(())
    }

    /// Liste les messages à évaluer, par ordre d'arrivée dans la file
    pub fn list_pending(&self) -> Result<Vec<MessageId>, MiraError> {
        let mut statement = self
            .sql()
            .prepare("SELECT message_id FROM rule_pending ORDER BY rowid")
            .map_err(storage_error)?;
        let ids = statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(storage_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(storage_error)?;
        Ok(ids.into_iter().map(MessageId::from_raw).collect())
    }

    /// Retire un message de la file une fois ses actions soumises
    pub fn remove_pending(&self, message: &MessageId) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "DELETE FROM rule_pending WHERE message_id = ?1",
                [message.as_str()],
            )
            .map_err(storage_error)?;
        Ok(())
    }
}
