//! Extraction des arguments d'une commande.

use crate::imap::parser::command::Command;
use crate::imap::parser::token::Token;
use crate::imap::response::error::ImapError;

/// Argument textuel à la position donnée, sinon BAD
pub(crate) fn text_arg(command: &Command, index: usize) -> Result<String, ImapError> {
    command
        .args
        .get(index)
        .and_then(Token::text)
        .ok_or_else(|| ImapError::bad("Missing argument"))
}

/// Argument en liste parenthésée à la position donnée, sinon BAD
pub(crate) fn list_arg(command: &Command, index: usize) -> Result<&[Token], ImapError> {
    command
        .args
        .get(index)
        .and_then(Token::list)
        .ok_or_else(|| ImapError::bad("Missing parenthesized list"))
}
