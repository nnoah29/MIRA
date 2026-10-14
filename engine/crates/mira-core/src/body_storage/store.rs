//! Étapes communes du rangement d'un corps : source, texte, pièces jointes, aperçu, état, index.

use mira_vocabulary::{ContentState, MessageSummary, MiraError};

use super::attachments::store_attachments;
use crate::mime::{ParsedMessage, preview_text};
use crate::ports::IdGenerator;
use crate::storage::Tx;

/// Range la source brute et tout ce qu'on en tire, dans la transaction de l'appelant
pub(crate) fn store_parsed_body(
    tx: &Tx<'_>,
    ids: &dyn IdGenerator,
    message: &MessageSummary,
    raw: &[u8],
    parsed: &ParsedMessage,
    addresses: &str,
) -> Result<(), MiraError> {
    tx.store_raw_source(&message.id, raw)?;
    tx.store_parsed(&message.id, &parsed.text_body, parsed.html_body.as_deref())?;
    store_attachments(tx, ids, &message.id, parsed)?;
    let has_files = parsed.attachments.iter().any(|part| !part.inline);
    tx.set_message_body_summary(&message.id, &preview_text(&parsed.text_body), has_files)?;
    tx.index_message(&message.id, &message.subject, addresses, &parsed.text_body)?;
    tx.set_content_state(&message.id, ContentState::BodyPresent)
}
