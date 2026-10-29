//! Secrets : clés d'API à préfixe connu, et valeur qui suit « mot de passe », « password », « mdp ».

use std::sync::LazyLock;

use regex::Regex;

const KEYS: &str = r"\b(?:sk-[A-Za-z0-9_\-]{16,}|ghp_[A-Za-z0-9]{20,}|AKIA[0-9A-Z]{16}|xox[baprs]-[A-Za-z0-9\-]{10,}|AIza[0-9A-Za-z_\-]{30,})";
const PASSWORD: &str =
    r"(?i)\b(?:mot de passe|password|passwd|pwd|mdp)[^\S\r\n]*[:=][^\S\r\n]*(\S+)";

static KEY_PATTERN: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(KEYS).ok());
static PASSWORD_PATTERN: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(PASSWORD).ok());

/// Emplacements (début, fin) des clés connues et des valeurs de mots de passe du texte.
pub(super) fn find(text: &str) -> Vec<(usize, usize)> {
    let mut spans = keys(text);
    spans.extend(passwords(text));
    spans
}

/// Clés d'API reconnues à leur préfixe.
fn keys(text: &str) -> Vec<(usize, usize)> {
    KEY_PATTERN.as_ref().map_or_else(Vec::new, |pattern| {
        pattern
            .find_iter(text)
            .map(|m| (m.start(), m.end()))
            .collect()
    })
}

/// Valeur seule qui suit le mot-clé, `:` ou `=` ; le mot-clé n'est pas masqué.
fn passwords(text: &str) -> Vec<(usize, usize)> {
    PASSWORD_PATTERN.as_ref().map_or_else(Vec::new, |pattern| {
        pattern
            .captures_iter(text)
            .filter_map(|captures| captures.get(1))
            .map(|value| (value.start(), value.end()))
            .collect()
    })
}
