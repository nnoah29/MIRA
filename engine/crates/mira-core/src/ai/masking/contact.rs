//! Coordonnées : adresses e-mail et numéros de téléphone (8 chiffres ou plus, `+` et séparateurs tolérés).

use std::sync::LazyLock;

use regex::Regex;

const EMAIL: &str = r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9\-]+(?:\.[A-Za-z0-9\-]+)*\.[A-Za-z]{2,}";
const PHONE: &str = r"(?:\+[0-9]|\b[0-9])(?:[ .\-]?[0-9]){7,14}\b";
const DATE: &str = r"^[0-9]{4}-[0-9]{2}-[0-9]{2}";

static EMAIL_PATTERN: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(EMAIL).ok());
static PHONE_PATTERN: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(PHONE).ok());
static DATE_PATTERN: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(DATE).ok());

/// Emplacements (début, fin) des adresses e-mail du texte.
pub(super) fn find_email(text: &str) -> Vec<(usize, usize)> {
    EMAIL_PATTERN.as_ref().map_or_else(Vec::new, |pattern| {
        pattern
            .find_iter(text)
            .map(|m| (m.start(), m.end()))
            .collect()
    })
}

/// Emplacements (début, fin) des numéros de téléphone du texte, hors dates et numéros de version.
pub(super) fn find_phone(text: &str) -> Vec<(usize, usize)> {
    PHONE_PATTERN.as_ref().map_or_else(Vec::new, |pattern| {
        pattern
            .find_iter(text)
            .filter(|m| looks_like_phone(m.as_str()))
            .map(|m| (m.start(), m.end()))
            .collect()
    })
}

/// Refuse les dates écrites avec des tirets et les suites à points dont un groupe suivant n'a qu'un chiffre.
fn looks_like_phone(candidate: &str) -> bool {
    let is_date = DATE_PATTERN
        .as_ref()
        .is_some_and(|date| date.is_match(candidate));
    let groups = candidate
        .split(|c: char| !c.is_ascii_digit())
        .filter(|group| !group.is_empty());
    let dotted = candidate.contains('.');
    !is_date && (!dotted || groups.skip(1).all(|group| group.len() >= 2))
}
