//! Description des pièces jointes et ressources incorporées.

use mail_parser::{Message, MessagePart, MimeHeaders, PartType};

use super::ids::clean_message_id;
use super::model::ParsedAttachment;

/// Type MIME en minuscules d'une partie, avec un type par défaut raisonnable.
fn mime_type_of(part: &MessagePart<'_>) -> String {
    match part.content_type() {
        Some(kind) => {
            let subtype = kind.c_subtype.as_deref().unwrap_or("octet-stream");
            format!("{}/{}", kind.c_type, subtype).to_ascii_lowercase()
        }
        None if matches!(part.body, PartType::Message(_)) => "message/rfc822".to_string(),
        None => "application/octet-stream".to_string(),
    }
}

/// Indique si la partie est incorporée (disposition `inline` ou ressource référencée).
fn is_inline(part: &MessagePart<'_>, content_id: &Option<String>) -> bool {
    match part.content_disposition() {
        Some(disposition) => disposition.is_inline(),
        None => content_id.is_some(),
    }
}

/// Décrit une partie jointe identifiée par son index.
fn describe(index: u32, part: &MessagePart<'_>) -> ParsedAttachment {
    let content_id = part.content_id().and_then(clean_message_id);
    ParsedAttachment {
        part_index: index,
        filename: part.attachment_name().map(str::to_string),
        mime_type: mime_type_of(part),
        size: part.len() as u64,
        inline: is_inline(part, &content_id),
        content_id,
    }
}

/// Liste les pièces jointes d'un message, dans l'ordre du message.
pub(crate) fn attachments_of(message: &Message<'_>) -> Vec<ParsedAttachment> {
    message
        .attachments
        .iter()
        .filter_map(|id| message.part(*id).map(|part| describe(*id, part)))
        .collect()
}
