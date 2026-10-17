//! Conversion d'une réponse FETCH en résultats typés du client.

use mira_vocabulary::RawMessageAttributes;

use super::dates::parse_internal_date;
use super::fetch::FetchData;
use super::render::render_raw;
use super::token::Token;
use crate::imap::types::{FetchedFlags, FetchedHeaders};

/// Drapeaux et numéro de modification d'un message.
pub(crate) fn fetched_flags(data: &FetchData) -> Option<FetchedFlags> {
    Some(FetchedFlags {
        uid: data.uid()?,
        flags: data.flags(),
        modseq: data.modseq(),
    })
}

/// Valeurs brutes des attributs supplémentaires demandés, dans l'ordre de la demande.
fn extra_attributes(data: &FetchData, extra: &[String]) -> RawMessageAttributes {
    let items = extra
        .iter()
        .filter_map(|name| {
            let key = name.to_ascii_uppercase();
            let value = data.get(&key)?;
            Some((key, render_raw(value)))
        })
        .collect();
    RawMessageAttributes { items }
}

/// En-têtes bruts, drapeaux, taille, date interne et attributs supplémentaires d'un message.
pub(crate) fn fetched_headers(data: &FetchData, extra: &[String]) -> Option<FetchedHeaders> {
    let header_bytes = match data.get("BODY[HEADER]") {
        Some(Token::Text(bytes)) => bytes.clone(),
        _ => Vec::new(),
    };
    Some(FetchedHeaders {
        uid: data.uid()?,
        flags: data.flags(),
        size: data.size(),
        internal_date: data
            .get("INTERNALDATE")
            .and_then(Token::text)
            .and_then(|text| parse_internal_date(&text)),
        modseq: data.modseq(),
        header_bytes,
        extra: extra_attributes(data, extra),
    })
}

/// Source complète du message (section `BODY[]`), reprise sans copie.
pub(crate) fn source_of(data: FetchData) -> Option<Vec<u8>> {
    data.items
        .into_iter()
        .find_map(|(name, token)| match (name.as_str(), token) {
            ("BODY[]", Token::Text(bytes)) => Some(bytes),
            _ => None,
        })
}
