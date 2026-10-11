//! Éléments demandés par FETCH.

use super::error::ParseError;
use super::fetch_section::parse_body;
use super::token::Token;

/// Section d'un message demandée par `BODY[...]`.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Section {
    Full,
    Header,
    Text,
    HeaderFields { names: Vec<String>, exclude: bool },
}

/// Demande de contenu `BODY[...]`, `BODY.PEEK[...]` ou `RFC822*`.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BodyRequest {
    pub section: Section,
    pub peek: bool,
    pub partial: Option<(usize, usize)>,
    pub label: String,
}

/// Élément FETCH reconnu par le faux serveur.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FetchItem {
    Uid,
    Flags,
    InternalDate,
    Size,
    Envelope,
    Body(BodyRequest),
    ModSeq,
    GmMsgId,
    GmThrId,
    GmLabels,
}

/// Lit la liste d'éléments (ou la macro ALL, FAST, FULL) d'une commande FETCH
pub(crate) fn parse_fetch_items(token: &Token) -> Result<Vec<FetchItem>, ParseError> {
    let tokens = token
        .list()
        .map_or_else(|| vec![token.clone()], <[Token]>::to_vec);
    let mut items = Vec::new();
    for entry in &tokens {
        items.extend(parse_entry(entry)?);
    }
    Ok(items)
}

/// Lit un élément ou une macro
fn parse_entry(token: &Token) -> Result<Vec<FetchItem>, ParseError> {
    let word = token
        .keyword()
        .ok_or_else(|| ParseError::new("invalid fetch item"))?;
    match word.as_str() {
        "ALL" | "FULL" => Ok(vec![
            FetchItem::Flags,
            FetchItem::InternalDate,
            FetchItem::Size,
            FetchItem::Envelope,
        ]),
        "FAST" => Ok(vec![
            FetchItem::Flags,
            FetchItem::InternalDate,
            FetchItem::Size,
        ]),
        other => parse_item(other).map(|item| vec![item]),
    }
}

/// Lit un élément simple
fn parse_item(word: &str) -> Result<FetchItem, ParseError> {
    match word {
        "UID" => Ok(FetchItem::Uid),
        "FLAGS" => Ok(FetchItem::Flags),
        "INTERNALDATE" => Ok(FetchItem::InternalDate),
        "RFC822.SIZE" => Ok(FetchItem::Size),
        "ENVELOPE" => Ok(FetchItem::Envelope),
        "MODSEQ" => Ok(FetchItem::ModSeq),
        "X-GM-MSGID" => Ok(FetchItem::GmMsgId),
        "X-GM-THRID" => Ok(FetchItem::GmThrId),
        "X-GM-LABELS" => Ok(FetchItem::GmLabels),
        other => parse_body(other),
    }
}
