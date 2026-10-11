//! Lecture d'un message du miroir local.

use mira_vocabulary::{MessageId, MessageSummary, MiraError};

use crate::storage::Database;
use crate::storage::repositories::messages::message_not_found;

/// Relit le résumé d'un message, ou échoue avec « message inconnu »
pub fn get_message(db: &Database, id: &MessageId) -> Result<MessageSummary, MiraError> {
    db.read(|tx| tx.get_message(id))?
        .ok_or_else(|| message_not_found(id))
}
