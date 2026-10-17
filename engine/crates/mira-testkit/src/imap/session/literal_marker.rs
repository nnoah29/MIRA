//! Détection d'un littéral annoncé en fin de ligne.

/// Taille et indicateur `+` d'un littéral `{n}` ou `{n+}` qui termine la ligne
pub(crate) fn literal_marker(line: &[u8]) -> Option<(usize, bool)> {
    let text = String::from_utf8_lossy(line);
    let inner = text.trim_end_matches(['\r', '\n']).strip_suffix('}')?;
    let spec = inner.get(inner.rfind('{')? + 1..)?;
    let (digits, plus) = match spec.strip_suffix('+') {
        Some(digits) => (digits, true),
        None => (spec, false),
    };
    digits.parse().ok().map(|size| (size, plus))
}
