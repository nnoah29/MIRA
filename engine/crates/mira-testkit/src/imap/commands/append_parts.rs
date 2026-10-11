//! Éléments d'une commande APPEND : drapeaux, date et message.

use crate::imap::date::clock::now_epoch;
use crate::imap::date::parse::parse_internal_date;
use crate::imap::parser::token::Token;
use crate::imap::response::error::ImapError;

/// Drapeaux, date interne et contenu du message à ajouter.
#[derive(Debug)]
pub(crate) struct AppendParts {
    pub flags: Vec<String>,
    pub date: i64,
    pub data: Vec<u8>,
}

/// Lit les arguments qui suivent le nom de la boîte
pub(crate) fn parse_append_parts(tokens: &[Token]) -> Result<AppendParts, ImapError> {
    let mut parts = AppendParts {
        flags: Vec::new(),
        date: now_epoch(),
        data: Vec::new(),
    };
    let mut has_data = false;
    for token in tokens {
        match token {
            Token::List(items) => parts.flags = items.iter().filter_map(Token::text).collect(),
            Token::Quoted(text) => {
                parts.date =
                    parse_internal_date(text).ok_or_else(|| ImapError::bad("Invalid date"))?;
            }
            Token::Literal(bytes) => {
                parts.data = bytes.clone();
                has_data = true;
            }
            Token::Atom(_) => return Err(ImapError::bad("Unexpected APPEND argument")),
        }
    }
    if has_data {
        Ok(parts)
    } else {
        Err(ImapError::bad("Missing message literal"))
    }
}
