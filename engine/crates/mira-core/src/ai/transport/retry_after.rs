//! Lecture de l'en-tête `retry-after` exprimé en secondes, plafonnée à une heure.

const MAX_RETRY_AFTER_SECS: u64 = 3600;

/// Donne le délai conseillé en secondes ; une date ou une valeur illisible ne donne rien.
pub(super) fn parse_retry_after(value: &str) -> Option<u64> {
    let digits = value.trim();
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let secs = digits.parse::<u64>().unwrap_or(MAX_RETRY_AFTER_SECS);
    Some(secs.min(MAX_RETRY_AFTER_SECS))
}
