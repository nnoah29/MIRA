//! Enregistrement des pièces jointes décrites par l'analyse d'un message.

use mira_vocabulary::{Attachment, AttachmentId, CacheState, MessageId, MiraError};

use crate::mime::ParsedMessage;
use crate::ports::IdGenerator;
use crate::storage::Tx;

/// Enregistre les pièces jointes décrites par l'analyse, si le message n'en a pas encore
pub(super) fn store_attachments(
    tx: &Tx<'_>,
    ids: &dyn IdGenerator,
    message: &MessageId,
    parsed: &ParsedMessage,
) -> Result<(), MiraError> {
    if !tx.list_attachments(message)?.is_empty() {
        return Ok(());
    }
    for part in &parsed.attachments {
        let attachment = Attachment {
            id: AttachmentId::from_raw(ids.next_id()),
            message: message.clone(),
            filename: part.filename.clone(),
            mime_type: part.mime_type.clone(),
            size: part.size,
            content_id: part.content_id.clone(),
            inline: part.inline,
            cache: CacheState::NotDownloaded,
        };
        tx.insert_attachment(&attachment, part.part_index)?;
    }
    Ok(())
}
