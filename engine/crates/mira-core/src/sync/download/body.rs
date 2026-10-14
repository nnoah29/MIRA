//! Enregistrement du corps d'un message : source brute compressée, texte, pièces jointes, aperçu, index.

use mira_vocabulary::{Attachment, AttachmentId, CacheState, ContentState, MessageId, MiraError};

use crate::mime::{ParsedAttachment, parse_message, preview_text};
use crate::storage::Tx;
use crate::sync::context::SyncContext;
use crate::sync::identity::address_text;

/// Pièce jointe décrite par l'analyse, pas encore téléchargée.
fn attachment_of(ctx: &SyncContext, message: &MessageId, part: &ParsedAttachment) -> Attachment {
    Attachment {
        id: AttachmentId::from_raw(ctx.new_id()),
        message: message.clone(),
        filename: part.filename.clone(),
        mime_type: part.mime_type.clone(),
        size: part.size,
        content_id: part.content_id.clone(),
        inline: part.inline,
        cache: CacheState::NotDownloaded,
    }
}

/// Enregistre la source brute et tout ce qui en est tiré ; sans effet si le corps est déjà présent.
pub fn store_body(
    tx: &Tx<'_>,
    ctx: &SyncContext,
    message: &MessageId,
    raw: &[u8],
) -> Result<bool, MiraError> {
    let Some(summary) = tx.get_message(message)? else {
        return Ok(false);
    };
    if summary.content == ContentState::BodyPresent {
        return Ok(false);
    }
    let parsed = parse_message(raw);
    tx.store_raw_source(message, raw)?;
    tx.store_parsed(message, &parsed.text_body, parsed.html_body.as_deref())?;
    for part in &parsed.attachments {
        tx.insert_attachment(&attachment_of(ctx, message, part), part.part_index)?;
    }
    let preview = preview_text(&parsed.text_body);
    let has_files = parsed.attachments.iter().any(|part| !part.inline);
    tx.set_message_body_summary(message, &preview, has_files)?;
    tx.index_message(
        message,
        &summary.subject,
        &address_text(&summary),
        &parsed.text_body,
    )?;
    tx.set_content_state(message, ContentState::BodyPresent)?;
    Ok(true)
}
