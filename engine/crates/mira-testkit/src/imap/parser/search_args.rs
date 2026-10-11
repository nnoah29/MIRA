//! Arguments des critères de recherche.

use super::error::ParseError;
use super::search::Criterion;
use super::search_keyword::parse_flag_or_set;
use super::token::Token;
use crate::imap::date::parse::parse_day;

/// Prend l'argument texte suivant
pub(crate) fn next_text(tokens: &[Token], pos: &mut usize) -> Result<String, ParseError> {
    let text = tokens.get(*pos).and_then(Token::text);
    *pos += 1;
    text.ok_or_else(|| ParseError::new("missing search argument"))
}

/// Prend l'argument numérique suivant
pub(crate) fn next_number(tokens: &[Token], pos: &mut usize) -> Result<u64, ParseError> {
    next_text(tokens, pos)?
        .parse()
        .map_err(|_| ParseError::new("invalid search number"))
}

/// Lit SINCE, BEFORE ou ON suivi d'une date
pub(crate) fn parse_date(
    word: &str,
    tokens: &[Token],
    pos: &mut usize,
) -> Result<Criterion, ParseError> {
    let day = parse_day(&next_text(tokens, pos)?)
        .ok_or_else(|| ParseError::new("invalid search date"))?;
    Ok(match word {
        "SINCE" => Criterion::Since(day),
        "BEFORE" => Criterion::Before(day),
        _ => Criterion::On(day),
    })
}

/// Lit MODSEQ avec ou sans nom d'attribut
pub(crate) fn parse_modseq(tokens: &[Token], pos: &mut usize) -> Result<Criterion, ParseError> {
    if matches!(tokens.get(*pos), Some(Token::Quoted(_))) {
        *pos += 2;
    }
    next_number(tokens, pos).map(Criterion::ModSeq)
}

/// Lit HEADER suivi d'un nom de champ et d'une chaîne à y trouver
pub(crate) fn parse_header(tokens: &[Token], pos: &mut usize) -> Result<Criterion, ParseError> {
    let field = next_text(tokens, pos)?;
    let needle = next_text(tokens, pos)?;
    Ok(Criterion::Header(field, needle))
}

/// Lit un critère textuel (TEXT, SUBJECT, FROM, TO, CC, BODY) ou renvoie vers drapeaux et ensembles
pub(crate) fn parse_text(
    word: &str,
    tokens: &[Token],
    pos: &mut usize,
) -> Result<Criterion, ParseError> {
    let build: fn(String) -> Criterion = match word {
        "HEADER" => return parse_header(tokens, pos),
        "TEXT" => Criterion::Text,
        "SUBJECT" => Criterion::Subject,
        "FROM" => Criterion::From,
        "TO" => Criterion::To,
        "CC" => Criterion::Cc,
        "BODY" => Criterion::Body,
        other => return parse_flag_or_set(other),
    };
    next_text(tokens, pos).map(build)
}
