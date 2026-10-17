//! Pièces jointes d'un message, lues dans le miroir local.

use mira_vocabulary::{Attachment, MessageId, MiraError};

use crate::storage::Database;
use crate::storage::repositories::messages::message_not_found;

/// Liste les pièces jointes d'un message, dans l'ordre des parties
pub fn list_attachments(db: &Database, id: &MessageId) -> Result<Vec<Attachment>, MiraError> {
    db.read(|tx| {
        tx.get_message(id)?.ok_or_else(|| message_not_found(id))?;
        tx.list_attachments(id)
    })
}
