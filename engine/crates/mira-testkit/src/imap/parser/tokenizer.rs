//! Découpage d'une commande brute en jetons.

use super::atoms::{parse_atom, parse_quoted};
use super::error::ParseError;
use super::literal::parse_literal;
use super::scanner::Cursor;
use super::token::Token;

/// Découpe toute la commande (étiquette comprise) en jetons
pub(crate) fn tokenize(data: &[u8]) -> Result<Vec<Token>, ParseError> {
    let mut cursor = Cursor::new(data);
    parse_sequence(&mut cursor, false)
}

/// Lit des jetons jusqu'à la fin ou jusqu'à la parenthèse fermante d'une liste
fn parse_sequence(cursor: &mut Cursor, nested: bool) -> Result<Vec<Token>, ParseError> {
    let mut tokens = Vec::new();
    loop {
        cursor.skip_blanks();
        match (cursor.peek(), nested) {
            (None, false) => return Ok(tokens),
            (None, true) => return Err(ParseError::new("unterminated list")),
            (Some(b')'), true) => {
                cursor.bump();
                return Ok(tokens);
            }
            (Some(b')'), false) => return Err(ParseError::new("unexpected closing parenthesis")),
            (Some(_), _) => tokens.push(parse_token(cursor)?),
        }
    }
}

/// Lit un jeton selon son premier caractère
fn parse_token(cursor: &mut Cursor) -> Result<Token, ParseError> {
    match cursor.peek() {
        Some(b'"') => parse_quoted(cursor).map(Token::Quoted),
        Some(b'{') => parse_literal(cursor).map(Token::Literal),
        Some(b'(') => {
            cursor.bump();
            parse_sequence(cursor, true).map(Token::List)
        }
        _ => Ok(Token::Atom(parse_atom(cursor))),
    }
}
