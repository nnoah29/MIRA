//! Mots-clés de recherche et leurs arguments.

use super::error::ParseError;
use super::search::{Criterion, parse_one};
use super::search_args::{next_number, next_text, parse_date, parse_modseq, parse_text};
use super::search_flag::flag_criterion;
use super::token::Token;

/// Lit le critère introduit par le mot-clé `word`
pub(crate) fn parse_keyword(
    word: &str,
    tokens: &[Token],
    pos: &mut usize,
) -> Result<Criterion, ParseError> {
    match word {
        "ALL" => Ok(Criterion::All),
        "NOT" => Ok(Criterion::Not(Box::new(parse_one(tokens, pos)?))),
        "OR" => Ok(Criterion::Or(
            Box::new(parse_one(tokens, pos)?),
            Box::new(parse_one(tokens, pos)?),
        )),
        "SINCE" | "BEFORE" | "ON" => parse_date(word, tokens, pos),
        "UID" => next_text(tokens, pos).map(Criterion::Uid),
        "MODSEQ" => parse_modseq(tokens, pos),
        "LARGER" => next_number(tokens, pos).map(Criterion::Larger),
        "SMALLER" => next_number(tokens, pos).map(Criterion::Smaller),
        "KEYWORD" => next_text(tokens, pos).map(|flag| Criterion::Flag(flag, true)),
        "UNKEYWORD" => next_text(tokens, pos).map(|flag| Criterion::Flag(flag, false)),
        other => parse_text(other, tokens, pos),
    }
}

/// Lit un critère fondé sur un drapeau ou un ensemble de messages
pub(crate) fn parse_flag_or_set(word: &str) -> Result<Criterion, ParseError> {
    if let Some(criterion) = flag_criterion(word) {
        return Ok(criterion);
    }
    if word.starts_with(|c: char| c.is_ascii_digit() || c == '*') {
        return Ok(Criterion::Seqset(word.to_string()));
    }
    Err(ParseError::new("unknown search key"))
}
