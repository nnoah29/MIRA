//! Lecture des éléments `BODY[...]` et `RFC822*` d'un FETCH.

use super::error::ParseError;
use super::fetch_items::{BodyRequest, FetchItem, Section};

/// Lit un élément de contenu (RFC822 ou BODY), avec section et fragment éventuels
pub(crate) fn parse_body(word: &str) -> Result<FetchItem, ParseError> {
    let (section, peek, partial, label) = match word {
        "RFC822" => (Section::Full, false, None, "RFC822".to_string()),
        "RFC822.HEADER" => (Section::Header, true, None, "RFC822.HEADER".to_string()),
        "RFC822.TEXT" => (Section::Text, false, None, "RFC822.TEXT".to_string()),
        other => return parse_bracketed(other),
    };
    Ok(FetchItem::Body(BodyRequest {
        section,
        peek,
        partial,
        label,
    }))
}

/// Lit `BODY[section]<origine.longueur>` ou la variante PEEK
fn parse_bracketed(word: &str) -> Result<FetchItem, ParseError> {
    let invalid = || ParseError::new("invalid body item");
    let peek = word.starts_with("BODY.PEEK[");
    if !peek && !word.starts_with("BODY[") {
        return Err(invalid());
    }
    let (open, close) = (
        word.find('[').ok_or_else(invalid)?,
        word.rfind(']').ok_or_else(invalid)?,
    );
    let text = word.get(open + 1..close).ok_or_else(invalid)?;
    let partial = parse_partial(word.get(close + 1..).unwrap_or(""))?;
    let origin = partial.map_or(String::new(), |(start, _)| format!("<{start}>"));
    Ok(FetchItem::Body(BodyRequest {
        section: parse_section(text)?,
        peek,
        partial,
        label: format!("BODY[{text}]{origin}"),
    }))
}

/// Lit un fragment `<origine.longueur>`
fn parse_partial(text: &str) -> Result<Option<(usize, usize)>, ParseError> {
    let Some(inner) = text
        .strip_prefix('<')
        .and_then(|rest| rest.strip_suffix('>'))
    else {
        return Ok(None);
    };
    let invalid = || ParseError::new("invalid partial range");
    let (start, length) = inner.split_once('.').ok_or_else(invalid)?;
    Ok(Some((
        start.parse().map_err(|_| invalid())?,
        length.parse().map_err(|_| invalid())?,
    )))
}

/// Lit la section : vide, HEADER, TEXT ou HEADER.FIELDS[.NOT] (champs)
fn parse_section(text: &str) -> Result<Section, ParseError> {
    match text {
        "" => Ok(Section::Full),
        "HEADER" => Ok(Section::Header),
        "TEXT" => Ok(Section::Text),
        other if other.starts_with("HEADER.FIELDS") => {
            let names = other
                .split_once('(')
                .map(|(_, rest)| rest.trim_end_matches(')'))
                .ok_or_else(|| ParseError::new("invalid header fields"))?;
            Ok(Section::HeaderFields {
                names: names.split_whitespace().map(str::to_string).collect(),
                exclude: other.starts_with("HEADER.FIELDS.NOT"),
            })
        }
        _ => Err(ParseError::new("unsupported body section")),
    }
}
