//! Parcours d'un motif qui repart un caractère plus loin quand un candidat est refusé.

use regex::Regex;

/// Cherche les candidats du motif ; `accept` rend la longueur retenue d'un candidat valide, `None` s'il est refusé.
pub(super) fn scan(
    pattern: &Regex,
    text: &str,
    accept: impl Fn(&str) -> Option<usize>,
) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut from = 0;
    while let Some(found) = pattern.find_at(text, from) {
        let start = found.start();
        match accept(found.as_str()).filter(|length| *length > 0) {
            Some(length) => {
                spans.push((start, start + length));
                from = start + length;
            }
            None => from = after_first_char(text, start),
        }
    }
    spans
}

/// Position qui suit le caractère commençant à `start`, toujours sur une frontière de caractère.
fn after_first_char(text: &str, start: usize) -> usize {
    text.get(start..)
        .and_then(|rest| rest.chars().next())
        .map_or(text.len(), |first| start + first.len_utf8())
}
