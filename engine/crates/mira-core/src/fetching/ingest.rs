//! Range une source téléchargée dans le miroir : source, texte, pièces jointes, état du contenu, index.

use mira_vocabulary::{
    Attachment, AttachmentId, CacheState, ContentState, MessageId, MessageSummary, MiraError,
};

use crate::mime::{HeaderSummary, ParsedMessage, parse_message};
use crate::ports::IdGenerator;
use crate::storage::Tx;

/// Texte indexé des adresses : nom et adresse de l'expéditeur, des destinataires et des copies
fn address_text(headers: &HeaderSummary) -> String {
    let entries = headers.from.iter().chain(&headers.to).chain(&headers.cc);
    entries
        .map(|entry| format!("{} {}", entry.name.as_deref().unwrap_or(""), entry.address))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Enregistre les pièces jointes décrites par l'analyse, si le message n'en a pas encore
fn store_attachments(
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

/// Range la source brute d'un message et tout ce qu'on en tire, dans la transaction de l'appelant
pub fn ingest_source(
    tx: &Tx<'_>,
    ids: &dyn IdGenerator,
    message: &MessageSummary,
    raw: &[u8],
) -> Result<(), MiraError> {
    let parsed = parse_message(raw);
    tx.store_raw_source(&message.id, raw)?;
    tx.store_parsed(&message.id, &parsed.text_body, parsed.html_body.as_deref())?;
    store_attachments(tx, ids, &message.id, &parsed)?;
    let visible = parsed.attachments.iter().any(|part| !part.inline);
    tx.set_message_has_attachments(&message.id, visible)?;
    tx.set_content_state(&message.id, ContentState::BodyPresent)?;
    let addresses = address_text(&parsed.headers);
    tx.index_message(&message.id, &message.subject, &addresses, &parsed.text_body)
}
