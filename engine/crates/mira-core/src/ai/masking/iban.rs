//! IBAN : forme, longueur du pays et contrôle modulo 97, sur le texte sans espaces ni tirets.

use std::sync::LazyLock;

use regex::Regex;

use super::scan::scan;

static PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"\b[A-Za-z]{2}[0-9]{2}(?:[ \-]?[A-Za-z0-9]){11,30}").ok());

/// Emplacements (début, fin) des IBAN valides du texte.
pub(super) fn find(text: &str) -> Vec<(usize, usize)> {
    match PATTERN.as_ref() {
        Some(pattern) => scan(pattern, text, accepted_length),
        None => Vec::new(),
    }
}

/// Longueur en octets du plus long début de candidat qui forme un IBAN valide.
fn accepted_length(candidate: &str) -> Option<usize> {
    let chars: Vec<(usize, char)> = candidate
        .char_indices()
        .filter(|(_, character)| character.is_ascii_alphanumeric())
        .collect();
    let country: String = chars.iter().take(2).map(|(_, c)| *c).collect();
    candidate_lengths(&country, chars.len())
        .into_iter()
        .find(|count| is_valid(chars.iter().take(*count).map(|(_, c)| *c)))
        .and_then(|count| chars.get(count - 1))
        .map(|(index, _)| index + 1)
}

/// Nombres de caractères à essayer : la longueur du pays si elle est connue, sinon de 34 à 15.
fn candidate_lengths(country: &str, available: usize) -> Vec<usize> {
    match (country_length(&country.to_ascii_uppercase()), available) {
        (Some(length), available) if length <= available => vec![length],
        (Some(_), _) => Vec::new(),
        (None, available) => (15..=available.min(34)).rev().collect(),
    }
}

/// Longueur normalisée d'un IBAN pour les pays courants.
fn country_length(country: &str) -> Option<usize> {
    match country {
        "NO" => Some(15),
        "BE" => Some(16),
        "NL" | "DK" | "FI" => Some(18),
        "LU" | "AT" => Some(20),
        "CH" => Some(21),
        "DE" | "GB" | "IE" => Some(22),
        "ES" | "SE" => Some(24),
        "PT" => Some(25),
        "FR" | "IT" | "MC" | "GR" => Some(27),
        "PL" => Some(28),
        _ => None,
    }
}

/// Contrôle modulo 97 : les quatre premiers caractères passent à la fin, les lettres valent 10 à 35.
fn is_valid(chars: impl Iterator<Item = char>) -> bool {
    let all: Vec<char> = chars.collect();
    if !(15..=34).contains(&all.len()) {
        return false;
    }
    let mut rotated = all.iter().skip(4).chain(all.iter().take(4));
    let remainder = rotated.try_fold(0u32, |rest, character| {
        let value = character.to_digit(36)?;
        let width = if value >= 10 { 100 } else { 10 };
        Some((rest * width + value) % 97)
    });
    remainder == Some(1)
}
