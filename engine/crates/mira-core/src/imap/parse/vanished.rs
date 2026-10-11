//! Réponse VANISHED (QRESYNC) : UID disparus de la boîte.

use super::token::Token;
use super::tokenizer::tokenize;
use super::uid_set::parse_uid_set;

/// Lit `VANISHED [(EARLIER)] ensemble` et rend les UID disparus.
pub(crate) fn parse_vanished(line: &[u8]) -> Option<Vec<u32>> {
    let mut tokens = tokenize(line).into_iter();
    (tokens.next()?.keyword()? == "VANISHED").then_some(())?;
    let set = tokens.find_map(|token| match token {
        Token::Atom(text) => Some(text),
        _ => None,
    })?;
    Some(parse_uid_set(&set))
}
