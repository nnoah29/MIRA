//! Contenu des sections BODY[...].

use crate::imap::message::fields::select_fields;
use crate::imap::message::split::{body_part, header_part};
use crate::imap::parser::fetch_items::{BodyRequest, Section};

/// Octets de la section demandée, fragment partiel compris
pub(crate) fn section_bytes(raw: &[u8], request: &BodyRequest) -> Vec<u8> {
    let whole = match &request.section {
        Section::Full => raw.to_vec(),
        Section::Header => header_part(raw).to_vec(),
        Section::Text => body_part(raw).to_vec(),
        Section::HeaderFields { names, exclude } => {
            select_fields(header_part(raw), names, *exclude)
        }
    };
    match request.partial {
        Some((start, length)) => whole.into_iter().skip(start).take(length).collect(),
        None => whole,
    }
}
