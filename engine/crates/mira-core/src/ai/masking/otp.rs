//! Code à usage unique : 4 à 8 chiffres à moins de 40 caractères après un mot comme « code » ou « OTP ».

use std::sync::LazyLock;

use regex::Regex;

const PATTERN: &str = r"(?i)\b(?:code|otp|vérification|verification|sécurité|security|one-time)\b[^0-9]{0,40}?\b([0-9]{4,8})\b";

static REGEX: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(PATTERN).ok());

/// Emplacements (début, fin) des chiffres de codes à usage unique ; des chiffres isolés ne comptent pas.
pub(super) fn find(text: &str) -> Vec<(usize, usize)> {
    REGEX.as_ref().map_or_else(Vec::new, |pattern| {
        pattern
            .captures_iter(text)
            .filter_map(|captures| captures.get(1))
            .map(|digits| (digits.start(), digits.end()))
            .collect()
    })
}
