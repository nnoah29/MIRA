//! Construction du résumé d'en-têtes à partir d'un message analysé.

use mail_parser::{HeaderName, HeaderValue, Message};
use mira_vocabulary::Timestamp;

use super::addresses::convert_address;
use super::ids::{clean_message_id, split_ids};
use super::model::HeaderSummary;

/// Date de l'en-tête `Date` en millisecondes, si elle est valide.
fn date_of(message: &Message<'_>) -> Option<Timestamp> {
    let date = message.date().filter(|date| date.is_valid())?;
    Some(Timestamp::from_millis(
        date.to_timestamp().saturating_mul(1000),
    ))
}

/// Identifiants contenus dans une valeur d'en-tête texte ou liste de textes.
fn ids_of(value: &HeaderValue<'_>) -> Vec<String> {
    match value {
        HeaderValue::Text(text) => split_ids(text),
        HeaderValue::TextList(list) => list.iter().flat_map(|text| split_ids(text)).collect(),
        _ => Vec::new(),
    }
}

/// Identifiants de tous les en-têtes `References`, dans l'ordre d'apparition.
fn references_of(message: &Message<'_>) -> Vec<String> {
    message
        .header_values(HeaderName::References)
        .flat_map(ids_of)
        .collect()
}

/// Résume les en-têtes d'un message déjà analysé.
pub(crate) fn summarize(message: &Message<'_>) -> HeaderSummary {
    HeaderSummary {
        from: convert_address(message.from()),
        to: convert_address(message.to()),
        cc: convert_address(message.cc()),
        bcc: convert_address(message.bcc()),
        reply_to: convert_address(message.reply_to()),
        subject: message.subject().unwrap_or_default().to_string(),
        date: date_of(message),
        message_id: message.message_id().and_then(clean_message_id),
        in_reply_to: ids_of(message.in_reply_to()).into_iter().next(),
        references: references_of(message),
    }
}
