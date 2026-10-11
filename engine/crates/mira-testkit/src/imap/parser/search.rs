//! Critères de recherche SEARCH.

use super::error::ParseError;
use super::search_keyword::parse_keyword;
use super::token::Token;

/// Critère de recherche reconnu par le faux serveur.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Criterion {
    All,
    Since(i64),
    Before(i64),
    On(i64),
    Uid(String),
    Seqset(String),
    Text(String),
    Subject(String),
    From(String),
    To(String),
    Cc(String),
    Body(String),
    Header(String, String),
    ModSeq(u64),
    Flag(String, bool),
    Larger(u64),
    Smaller(u64),
    Not(Box<Criterion>),
    Or(Box<Criterion>, Box<Criterion>),
    And(Vec<Criterion>),
}

/// Recherche analysée : critères et options RETURN éventuelles.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SearchQuery {
    pub criteria: Vec<Criterion>,
    pub returns: Option<Vec<String>>,
}

/// Analyse les arguments d'une commande SEARCH
pub(crate) fn parse_search(args: &[Token]) -> Result<SearchQuery, ParseError> {
    let mut pos = 0;
    let mut returns = None;
    if args.first().and_then(Token::keyword).as_deref() == Some("RETURN") {
        let options = args.get(1).and_then(Token::list);
        returns = Some(
            options
                .unwrap_or(&[])
                .iter()
                .filter_map(Token::keyword)
                .collect(),
        );
        pos = 2;
    }
    if args.get(pos).and_then(Token::keyword).as_deref() == Some("CHARSET") {
        pos += 2;
    }
    Ok(SearchQuery {
        criteria: parse_all(args, &mut pos)?,
        returns,
    })
}

/// Lit tous les critères restants
pub(crate) fn parse_all(tokens: &[Token], pos: &mut usize) -> Result<Vec<Criterion>, ParseError> {
    let mut criteria = Vec::new();
    while *pos < tokens.len() {
        criteria.push(parse_one(tokens, pos)?);
    }
    Ok(criteria)
}

/// Lit un critère, y compris une liste parenthésée
pub(crate) fn parse_one(tokens: &[Token], pos: &mut usize) -> Result<Criterion, ParseError> {
    let token = tokens
        .get(*pos)
        .ok_or_else(|| ParseError::new("missing search criterion"))?;
    *pos += 1;
    if let Some(inner) = token.list() {
        return Ok(Criterion::And(parse_all(inner, &mut 0)?));
    }
    let word = token
        .keyword()
        .ok_or_else(|| ParseError::new("invalid search criterion"))?;
    parse_keyword(&word, tokens, pos)
}
