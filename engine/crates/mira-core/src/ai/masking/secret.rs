//! Secrets : clés à préfixe connu, valeur d'un mot de passe, secret par contexte, JWT et clé PEM.

use std::sync::LazyLock;

use regex::Regex;

use super::{context, structured};

const CLOSING: &[char] = &[
    '.', ',', ';', ':', '!', '?', ')', ']', '}', '"', '\'', '»', '”', '’',
];

const KEYS: &str = r"\b(?:sk-[A-Za-z0-9_\-]{16,}|ghp_[A-Za-z0-9]{20,}|AKIA[0-9A-Z]{16}|xox[baprs]-[A-Za-z0-9\-]{10,}|AIza[0-9A-Za-z_\-]{30,}|glpat-[A-Za-z0-9_\-]{20,}|[sr]k_live_[A-Za-z0-9]{16,}|npm_[A-Za-z0-9]{36})";
const PASSWORD: &str =
    r#"(?i)\b(?:mot de passe|password|passwd|pwd|mdp)[^\S\r\n]*[:=][^\S\r\n]*["'«“‘]?(\S+)"#;

static KEY_PATTERN: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(KEYS).ok());
static PASSWORD_PATTERN: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(PASSWORD).ok());

/// Emplacements (début, fin) des secrets du texte, fusionnés quand deux détecteurs se recouvrent.
pub(super) fn find(text: &str) -> Vec<(usize, usize)> {
    let mut spans = keys(text);
    spans.extend(passwords(text));
    spans.extend(context::find(text));
    spans.extend(structured::find(text));
    merged(spans)
}

/// Fusionne les emplacements qui se touchent ou se recouvrent, triés par début.
fn merged(mut spans: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    spans.sort_unstable();
    let mut result: Vec<(usize, usize)> = Vec::with_capacity(spans.len());
    for (start, end) in spans {
        match result.last_mut() {
            Some(last) if start < last.1 => last.1 = last.1.max(end),
            _ => result.push((start, end)),
        }
    }
    result
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

/// Valeur seule qui suit le mot-clé, `:` ou `=`, sans ponctuation de fin ; le mot-clé n'est pas masqué.
fn passwords(text: &str) -> Vec<(usize, usize)> {
    PASSWORD_PATTERN.as_ref().map_or_else(Vec::new, |pattern| {
        pattern
            .captures_iter(text)
            .filter_map(|captures| captures.get(1))
            .map(|value| (value.start(), value.start() + trimmed(value.as_str()).len()))
            .collect()
    })
}

/// Valeur sans ponctuation ni guillemet de fin : ils appartiennent à la phrase, pas au mot de passe.
fn trimmed(value: &str) -> &str {
    value.trim_end_matches(CLOSING)
}
