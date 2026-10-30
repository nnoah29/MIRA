//! Code à usage unique : 4 à 8 chiffres à moins de 40 caractères après un mot comme « code » ou « OTP ».

use std::sync::LazyLock;

use regex::Regex;

const PATTERN: &str = r"(?i)\b(?:code|otp|vérification|verification|sécurité|security|one-time)\b([^0-9]{0,40}?)\b([0-9]{4,8})\b";
const POSTAL_AFTER: &str = r"(?i)\b(?:postal|postale|zip)\b";
const POSTAL_BEFORE: &str = r"(?i)\b(?:postal|postale|zip)[^\S\r\n]*$";

static REGEX: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(PATTERN).ok());
static AFTER: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(POSTAL_AFTER).ok());
static BEFORE: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(POSTAL_BEFORE).ok());

/// Emplacements (début, fin) des chiffres de codes à usage unique ; des chiffres isolés ou un code postal ne comptent pas.
pub(super) fn find(text: &str) -> Vec<(usize, usize)> {
    REGEX.as_ref().map_or_else(Vec::new, |pattern| {
        pattern
            .captures_iter(text)
            .filter_map(|captures| Some((captures.get(0)?, captures.get(1)?, captures.get(2)?)))
            .filter(|(whole, gap, _)| !is_postal(text, whole.start(), gap.as_str()))
            .map(|(_, _, digits)| (digits.start(), digits.end()))
            .collect()
    })
}

/// Dit si le mot-clé est « code postal », « postal code » ou « zip code » : un code postal n'est pas un secret.
fn is_postal(text: &str, keyword_start: usize, gap: &str) -> bool {
    let after = AFTER.as_ref().is_some_and(|pattern| pattern.is_match(gap));
    let before = BEFORE
        .as_ref()
        .is_some_and(|pattern| pattern.is_match(text.get(..keyword_start).unwrap_or_default()));
    after || before
}
