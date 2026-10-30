//! Secret générique : une valeur longue et aléatoire après un mot-clé comme « token » ou « api_key ».

use std::sync::LazyLock;

use regex::Regex;

use super::entropy::shannon;

const KEYWORDS: &str =
    r"api[_-]?key|apikey|secret|token|authorization|access[_-]?key|private[_-]?key|clé|jeton";
const VALUE: &str = r#"["']?([A-Za-z0-9+/=_.-]{20,})"#;
const MIN_LENGTH: usize = 20;
const BASE64_LIMIT: f64 = 4.5;
const HEX_LIMIT: f64 = 3.0;
const HEX_MIN_LENGTH: usize = 32;

static KEYED: LazyLock<Option<Regex>> = LazyLock::new(|| {
    let pattern =
        format!(r#"(?i)(?:{KEYWORDS})[A-Za-z0-9_-]{{0,20}}["']?[^\S\r\n]*[:=][^\S\r\n]*{VALUE}"#);
    Regex::new(&pattern).ok()
});
static BEARER: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(&format!(r"(?i)\bbearer[^\S\r\n]+{VALUE}")).ok());

/// Valeurs (début, fin) qui suivent un mot-clé et dont l'entropie atteint le seuil ; sans mot-clé, rien.
pub(super) fn find(text: &str) -> Vec<(usize, usize)> {
    [&KEYED, &BEARER]
        .into_iter()
        .filter_map(|pattern| pattern.as_ref())
        .flat_map(|pattern| pattern.captures_iter(text))
        .filter_map(|captures| captures.get(1))
        .map(|value| (value.start(), value.as_str().trim_end_matches('.')))
        .filter(|(_, value)| value.len() >= MIN_LENGTH && is_random(value))
        .map(|(start, value)| (start, start + value.len()))
        .collect()
}

/// Dit si la valeur ressemble à un secret : ni UUID, entropie au-dessus du seuil de son alphabet.
fn is_random(value: &str) -> bool {
    if is_uuid(value) {
        return false;
    }
    let hex = value.len() >= HEX_MIN_LENGTH && value.chars().all(|c| c.is_ascii_hexdigit());
    shannon(value) >= if hex { HEX_LIMIT } else { BASE64_LIMIT }
}

/// Dit si la valeur est un UUID (8-4-4-4-12 chiffres hexadécimaux).
fn is_uuid(value: &str) -> bool {
    let groups: Vec<&str> = value.split('-').collect();
    let sizes: Vec<usize> = groups.iter().map(|group| group.len()).collect();
    sizes == [8, 4, 4, 4, 12]
        && groups
            .iter()
            .all(|group| group.chars().all(|c| c.is_ascii_hexdigit()))
}
