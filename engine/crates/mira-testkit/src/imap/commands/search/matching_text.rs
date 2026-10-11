//! Critères textuels : sujet, expéditeur, corps, champ d'en-tête quelconque.

use crate::imap::message::fields::header_value;
use crate::imap::message::split::body_part;
use crate::imap::parser::search::Criterion;
use crate::imap::store::data::Entry;

/// Recherche insensible à la casse d'un texte dans des octets
fn contains(bytes: &[u8], needle: &str) -> bool {
    let haystack = String::from_utf8_lossy(bytes).to_lowercase();
    haystack.contains(&needle.to_lowercase())
}

/// Recherche insensible à la casse dans la valeur d'un champ d'en-tête
fn field_contains(raw: &[u8], field: &str, needle: &str) -> bool {
    header_value(raw, field).is_some_and(|value| contains(value.as_bytes(), needle))
}

/// Évalue un critère textuel sur un message
pub(crate) fn matches_text(criterion: &Criterion, entry: &Entry) -> bool {
    let raw = &entry.message.raw;
    match criterion {
        Criterion::Text(needle) => contains(raw, needle),
        Criterion::Body(needle) => contains(body_part(raw), needle),
        Criterion::Subject(needle) => field_contains(raw, "Subject", needle),
        Criterion::From(needle) => field_contains(raw, "From", needle),
        Criterion::To(needle) => field_contains(raw, "To", needle),
        Criterion::Cc(needle) => field_contains(raw, "Cc", needle),
        Criterion::Header(field, needle) => field_contains(raw, field, needle),
        _ => false,
    }
}
