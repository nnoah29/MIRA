//! Comparaison de texte des conditions : casse Unicode ignorée, regex bornée.

use mira_vocabulary::{TextMatcher, TextOp};
use regex::{Regex, RegexBuilder};

/// Taille maximale d'une regex compilée, en octets.
pub(crate) const REGEX_SIZE_LIMIT: usize = 1 << 20;

/// Vrai si au moins une valeur du filtre correspond au texte.
pub fn matches_text(matcher: &TextMatcher, value: &str) -> bool {
    let lowered = value.to_lowercase();
    matcher
        .values
        .iter()
        .any(|candidate| matches_candidate(matcher.op, candidate, value, &lowered))
}

/// Applique l'opérateur à une valeur du filtre ; `lowered` est le texte en minuscules.
fn matches_candidate(op: TextOp, candidate: &str, value: &str, lowered: &str) -> bool {
    let wanted = candidate.to_lowercase();
    match op {
        TextOp::Contains => lowered.contains(&wanted),
        TextOp::Equals | TextOp::InList => lowered == wanted,
        TextOp::StartsWith => lowered.starts_with(&wanted),
        TextOp::EndsWith => lowered.ends_with(&wanted),
        TextOp::Regex => regex_matches(candidate, value),
    }
}

/// Compile une regex insensible à la casse, plafonnée en taille ; partagée avec la validation.
pub(crate) fn compile_regex(pattern: &str) -> Result<Regex, regex::Error> {
    RegexBuilder::new(pattern)
        .case_insensitive(true)
        .size_limit(REGEX_SIZE_LIMIT)
        .build()
}

/// Vrai si la regex, insensible à la casse, se trouve dans le texte ; une regex invalide est fausse.
fn regex_matches(pattern: &str, value: &str) -> bool {
    compile_regex(pattern).is_ok_and(|regex| regex.is_match(value))
}
