//! Détection d'un littéral `{n}` en fin de ligne.

/// Taille annoncée par un littéral en fin de ligne physique, s'il y en a un.
pub(crate) fn literal_size(line: &[u8]) -> Option<usize> {
    let inner = line.strip_suffix(b"}")?;
    let open = inner.iter().rposition(|byte| *byte == b'{')?;
    let digits = &inner[open + 1..];
    let digits = digits.strip_suffix(b"+").unwrap_or(digits);
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
        return None;
    }
    std::str::from_utf8(digits).ok()?.parse().ok()
}
