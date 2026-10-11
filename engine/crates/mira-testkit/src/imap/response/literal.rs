//! Écriture des littéraux IMAP.

/// Littéral `{n}` suivi de ses octets
pub(crate) fn literal(data: &[u8]) -> Vec<u8> {
    let mut out = format!("{{{}}}\r\n", data.len()).into_bytes();
    out.extend_from_slice(data);
    out
}
