//! Séparation des en-têtes et du corps d'un message brut.

/// Position de la première occurrence de `needle`
fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Position où commence le corps, juste après la ligne vide
pub(crate) fn body_start(raw: &[u8]) -> usize {
    find(raw, b"\r\n\r\n")
        .map(|position| position + 4)
        .or_else(|| find(raw, b"\n\n").map(|position| position + 2))
        .unwrap_or(raw.len())
}

/// En-têtes, ligne vide comprise
pub(crate) fn header_part(raw: &[u8]) -> &[u8] {
    &raw[..body_start(raw)]
}

/// Corps du message
pub(crate) fn body_part(raw: &[u8]) -> &[u8] {
    &raw[body_start(raw)..]
}
