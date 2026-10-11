//! Analyse d'un message complet et extraction du contenu d'une pièce jointe.

use mail_parser::MessageParser;

use super::attachments::attachments_of;
use super::bodies::{html_of, text_of};
use super::model::{ParsedAttachment, ParsedMessage};
use super::summary::summarize;
use super::unsubscribe::unsubscribe_from;

const MAX_PARSED_BYTES: usize = 50 * 1024 * 1024;
const MAX_HEADER_BLOCK: usize = 1024 * 1024;

/// Bloc d'en-têtes : jusqu'à la première ligne vide, avec une borne de taille.
fn header_block(raw: &[u8]) -> &[u8] {
    let limit = raw.len().min(MAX_HEADER_BLOCK);
    let window = &raw[..limit];
    let end = window
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .or_else(|| window.windows(2).position(|w| w == b"\n\n"));
    &window[..end.map_or(limit, |at| (at + 4).min(limit))]
}

/// Message trop gros : en-têtes seuls, corps vide, une pièce jointe non décrite.
fn oversized(raw: &[u8]) -> ParsedMessage {
    let message = MessageParser::default().parse_headers(header_block(raw));
    ParsedMessage {
        headers: message.as_ref().map(summarize).unwrap_or_default(),
        unsubscribe: message.as_ref().and_then(unsubscribe_from),
        attachments: vec![ParsedAttachment {
            part_index: 0,
            filename: None,
            mime_type: "application/octet-stream".to_string(),
            size: raw.len() as u64,
            content_id: None,
            inline: false,
        }],
        ..ParsedMessage::default()
    }
}

/// Analyse un message brut sans jamais échouer ; un message illisible donne un objet vide.
pub fn parse_message(raw: &[u8]) -> ParsedMessage {
    if raw.len() > MAX_PARSED_BYTES {
        return oversized(raw);
    }
    let Some(message) = MessageParser::default().parse(raw) else {
        return ParsedMessage::default();
    };
    ParsedMessage {
        headers: summarize(&message),
        text_body: text_of(&message),
        html_body: html_of(&message),
        attachments: attachments_of(&message),
        unsubscribe: unsubscribe_from(&message),
    }
}

/// Contenu décodé d'une partie jointe, ou `None` si l'index est inconnu ou le message trop gros.
pub fn attachment_bytes(raw: &[u8], part_index: u32) -> Option<Vec<u8>> {
    if raw.len() > MAX_PARSED_BYTES {
        return None;
    }
    let message = MessageParser::default().parse(raw)?;
    let part = message.part(part_index)?;
    Some(part.contents().to_vec())
}
