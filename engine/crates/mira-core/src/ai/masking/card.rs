//! Carte bancaire : 13 à 19 chiffres séparés au plus d'un espace ou d'un tiret, avec contrôle de Luhn.

use std::sync::LazyLock;

use regex::Regex;

use super::scan::scan;

static PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"\b[0-9](?:[ \-]?[0-9]){12,18}\b").ok());

/// Emplacements (début, fin) des numéros de carte valides du texte.
pub(super) fn find(text: &str) -> Vec<(usize, usize)> {
    match PATTERN.as_ref() {
        Some(pattern) => scan(pattern, text, accepted_length),
        None => Vec::new(),
    }
}

/// Longueur du candidat entier s'il passe le contrôle de Luhn.
fn accepted_length(candidate: &str) -> Option<usize> {
    let digits: Vec<u32> = candidate.chars().filter_map(|c| c.to_digit(10)).collect();
    (13..=19)
        .contains(&digits.len())
        .then(|| luhn_sum(&digits))
        .filter(|sum| sum % 10 == 0)
        .map(|_| candidate.len())
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
