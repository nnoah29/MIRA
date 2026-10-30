//! Carte bancaire : 13 à 19 chiffres, groupes séparés d'un seul séparateur, avec contrôle de Luhn.

use std::sync::LazyLock;

use regex::Regex;

use super::scan::scan;

static PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"\b[0-9](?:(?:\r\n|[\s\u{a0}\u{202f}.\-])?[0-9]){12,18}\b").ok());

/// Emplacements (début, fin) des numéros de carte valides du texte.
pub(super) fn find(text: &str) -> Vec<(usize, usize)> {
    match PATTERN.as_ref() {
        Some(pattern) => scan(pattern, text, accepted_length),
        None => Vec::new(),
    }
}

/// Longueur du plus long début du candidat, terminé en fin de groupe, qui passe le contrôle de Luhn.
fn accepted_length(candidate: &str) -> Option<usize> {
    let digits: Vec<u32> = candidate.chars().filter_map(|c| c.to_digit(10)).collect();
    group_ends(candidate)
        .into_iter()
        .rev()
        .filter(|(count, _)| (13..=19).contains(count))
        .find(|(count, _)| luhn_sum(digits.get(..*count).unwrap_or_default()).is_multiple_of(10))
        .map(|(_, end)| end)
}

/// Pour chaque fin de groupe de chiffres : le nombre de chiffres lus et la position d'octet de la fin.
fn group_ends(candidate: &str) -> Vec<(usize, usize)> {
    let mut count = 0;
    let mut ends = Vec::new();
    let mut chars = candidate.char_indices().peekable();
    while let Some((index, character)) = chars.next() {
        if !character.is_ascii_digit() {
            continue;
        }
        count += 1;
        if chars.peek().is_none_or(|(_, next)| !next.is_ascii_digit()) {
            ends.push((count, index + 1));
        }
    }
    ends
}

/// Somme de Luhn : un chiffre sur deux, en partant de la droite, est doublé.
fn luhn_sum(digits: &[u32]) -> u32 {
    digits
        .iter()
        .rev()
        .enumerate()
        .map(|(index, digit)| match index % 2 {
            1 => digit * 2 / 10 + digit * 2 % 10,
            _ => *digit,
        })
        .sum()
}
