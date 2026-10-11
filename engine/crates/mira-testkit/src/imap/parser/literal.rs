//! Lecture d'un littéral `{n}` ou `{n+}` déjà présent dans la commande.

use super::error::ParseError;
use super::scanner::Cursor;

/// Lit le littéral qui commence à l'accolade ouvrante
pub(crate) fn parse_literal(cursor: &mut Cursor) -> Result<Vec<u8>, ParseError> {
    cursor.bump();
    let mut digits = String::new();
    while let Some(byte) = cursor.bump() {
        match byte {
            b'}' => break,
            b'+' => {}
            other => digits.push(char::from(other)),
        }
    }
    let size: usize = digits
        .parse()
        .map_err(|_| ParseError::new("invalid literal size"))?;
    cursor.skip_line_end();
    let data = cursor
        .take(size)
        .ok_or_else(|| ParseError::new("literal data missing"))?;
    Ok(data.to_vec())
}
