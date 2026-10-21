//! Lecture des atomes et des chaînes entre guillemets.

use super::error::ParseError;
use super::scanner::Cursor;

/// Lit un atome ; les crochets (sections FETCH) en font partie
pub(crate) fn parse_atom(cursor: &mut Cursor) -> String {
    let mut bytes = Vec::new();
    while let Some(byte) = cursor.peek() {
        if matches!(byte, b' ' | b'\t' | b'(' | b')' | b'\r' | b'\n') {
            break;
        }
        bytes.push(byte);
        cursor.bump();
        if byte == b'[' {
            take_bracket(cursor, &mut bytes);
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Copie le contenu d'une section entre crochets, espaces et parenthèses compris
fn take_bracket(cursor: &mut Cursor, bytes: &mut Vec<u8>) {
    while let Some(byte) = cursor.bump() {
        bytes.push(byte);
        if byte == b']' {
            break;
        }
    }
}

/// Lit une chaîne entre guillemets avec ses échappements
pub(crate) fn parse_quoted(cursor: &mut Cursor) -> Result<String, ParseError> {
    cursor.bump();
    let mut bytes = Vec::new();
    loop {
        match cursor.bump() {
            None => return Err(ParseError::new("unterminated quoted string")),
            Some(b'\\') => bytes.extend(cursor.bump()),
            Some(b'"') => break,
            Some(byte) if !byte.is_ascii() => {
                return Err(ParseError::new("non-ASCII byte in quoted string"));
            }
            Some(byte) => bytes.push(byte),
        }
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}
