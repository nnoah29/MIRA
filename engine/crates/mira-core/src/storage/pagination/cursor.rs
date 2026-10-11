//! Curseur de pagination opaque : position de tri encodée en hexadécimal.

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{Cursor, ErrorCategory, MiraError};

const TAG: &str = "mira1";

/// Encode une position de tri (clé, identifiant) en curseur opaque
pub(crate) fn encode_cursor(sort_key: i64, id: &str) -> Cursor {
    Cursor(hex_encode(format!("{TAG}|{sort_key}|{id}").as_bytes()))
}

/// Décode un curseur opaque en position de tri, ou refuse un curseur invalide
pub(crate) fn decode_cursor(cursor: &Cursor) -> Result<(i64, String), MiraError> {
    let bytes = hex_decode(&cursor.0).ok_or_else(invalid_cursor)?;
    let text = String::from_utf8(bytes).map_err(|_| invalid_cursor())?;
    let mut parts = text.splitn(3, '|');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(TAG), Some(key), Some(id)) => {
            let key = key.parse::<i64>().map_err(|_| invalid_cursor())?;
            Ok((key, id.to_string()))
        }
        _ => Err(invalid_cursor()),
    }
}

/// Écrit des octets en hexadécimal minuscule
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Relit un texte hexadécimal, ou rien s'il est mal formé
fn hex_decode(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|start| u8::from_str_radix(text.get(start..start + 2)?, 16).ok())
        .collect()
}

/// Erreur « curseur invalide »
fn invalid_cursor() -> MiraError {
    MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::CURSOR,
        "curseur de pagination invalide",
    )
}
