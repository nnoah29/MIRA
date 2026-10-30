//! Secrets à structure fixe : jeton JWT (trois segments base64url) et bloc de clé privée PEM.
//! Un bloc PEM sans ligne de fin est masqué jusqu'à la fin du texte.

use std::sync::LazyLock;

use regex::Regex;

const JWT: &str = r"\beyJ[A-Za-z0-9_-]{7,}\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+";
const PEM: &str =
    r"(?s)-----BEGIN [A-Z ]*PRIVATE KEY-----.*?(?:-----END [A-Z ]*PRIVATE KEY-----|\z)";

static JWT_PATTERN: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(JWT).ok());
static PEM_PATTERN: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(PEM).ok());

/// Emplacements (début, fin) des jetons JWT et des blocs de clé privée PEM du texte.
pub(super) fn find(text: &str) -> Vec<(usize, usize)> {
    [&JWT_PATTERN, &PEM_PATTERN]
        .into_iter()
        .filter_map(|pattern| pattern.as_ref())
        .flat_map(|pattern| pattern.find_iter(text))
        .map(|found| (found.start(), found.end()))
        .collect()
}
