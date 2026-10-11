//! Lecture inverse : identifiant serveur stable connu pour un message.

use mira_vocabulary::{AccountId, MessageId, MiraError};
use rusqlite::{OptionalExtension, params};

use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Donne l'identifiant serveur stable d'un message (X-GM-MSGID, EMAILID…), s'il en a un
    pub fn stable_server_id_of(
        &self,
        account: &AccountId,
        message: &MessageId,
    ) -> Result<Option<String>, MiraError> {
        self.sql()
            .query_row(
                "SELECT value FROM server_ids WHERE account_id = ?1 AND kind = 'stable' AND message_id = ?2 ORDER BY value LIMIT 1",
                params![account.as_str(), message.as_str()],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(storage_error)
    }
}
