//! Ensembles de messages (`1:*`, `5:10`, `3,7,9`) résolus contre une boîte.

use super::error::ParseError;

/// Lit une borne numérique ou `*`
fn parse_bound(text: &str, star: u32) -> Result<u32, ParseError> {
    if text == "*" {
        return Ok(star);
    }
    text.parse::<u32>()
        .map_err(|_| ParseError::new("invalid message set"))
}

/// Lit un élément `a`, `a:b` ou `a:*` en bornes ordonnées
fn parse_range(part: &str, star: u32) -> Result<(u32, u32), ParseError> {
    let (first, last) = part.split_once(':').unwrap_or((part, part));
    let (low, high) = (parse_bound(first, star)?, parse_bound(last, star)?);
    Ok((low.min(high), low.max(high)))
}

/// Indique si la clé appartient à l'une des bornes
fn in_ranges(ranges: &[(u32, u32)], key: u32) -> bool {
    ranges.iter().any(|(low, high)| key >= *low && key <= *high)
}

/// UID (triés) des messages désignés par l'ensemble ; numéros de séquence si `by_uid` est faux
pub(crate) fn resolve_set(text: &str, ids: &[u32], by_uid: bool) -> Result<Vec<u32>, ParseError> {
    let star = if by_uid {
        ids.last().copied().unwrap_or(0)
    } else {
        ids.len() as u32
    };
    let ranges = text
        .split(',')
        .map(|part| parse_range(part, star))
        .collect::<Result<Vec<_>, _>>()?;
    let chosen = ids.iter().enumerate().filter(|(index, uid)| {
        let key = if by_uid { **uid } else { *index as u32 + 1 };
        in_ranges(&ranges, key)
    });
    Ok(chosen.map(|(_, uid)| *uid).collect())
}
