//! Range une source téléchargée à la demande dans le miroir, avec l'analyse faite hors transaction.

use mira_vocabulary::{MessageSummary, MiraError};

use crate::body_storage::store_parsed_body;
use crate::mime::{HeaderSummary, ParsedMessage};
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

/// Range la source brute d'un message et son analyse, dans la transaction de l'appelant
pub fn ingest_source(
    tx: &Tx<'_>,
    ids: &dyn IdGenerator,
    message: &MessageSummary,
    raw: &[u8],
    parsed: &ParsedMessage,
) -> Result<(), MiraError> {
    let addresses = address_text(&parsed.headers);
    store_parsed_body(tx, ids, message, raw, parsed, &addresses)
}
