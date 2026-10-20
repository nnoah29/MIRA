//! Fusion de deux messages qui n'en font qu'un : le jumeau rejoint le message conservé.

use mira_vocabulary::{MessageId, MiraError};

use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Rattache apparitions et correspondances serveur du jumeau au message conservé, puis supprime le jumeau
    pub fn merge_twin_into(&self, twin: &MessageId, kept: &MessageId) -> Result<(), MiraError> {
        let moved = [
            "UPDATE appearances SET message_id = ?2 WHERE message_id = ?1",
            "UPDATE server_ids SET message_id = ?2 WHERE message_id = ?1",
        ];
        for statement in moved {
            self.sql()
                .execute(statement, [twin.as_str(), kept.as_str()])
                .map_err(storage_error)?;
        }
        self.delete_message(twin)
    }
}
