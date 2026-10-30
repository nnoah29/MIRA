//! Découpe d'un flux d'octets en lignes complètes (`\n`, `\r\n` ou `\r`), décodées en UTF-8 par ligne.

/// Tampon d'octets en attente d'une fin de ligne.
#[derive(Debug, Default)]
pub(super) struct LineBuffer {
    bytes: Vec<u8>,
}

impl LineBuffer {
    /// Ajoute des octets et rend toutes les lignes désormais complètes.
    pub(super) fn push(&mut self, chunk: &[u8]) -> Vec<String> {
        self.bytes.extend_from_slice(chunk);
        let mut lines = Vec::new();
        let mut start = 0;
        while let Some((end, next)) = find_line_end(&self.bytes[start..]) {
            lines.push(decode(&self.bytes[start..start + end]));
            start += next;
        }
        self.bytes.drain(..start);
        lines
    }

    /// Vide le tampon en fin de flux : dernière ligne sans fin de ligne, s'il en reste une.
    pub(super) fn take_rest(&mut self) -> Option<String> {
        let mut rest = std::mem::take(&mut self.bytes);
        if rest.last() == Some(&b'\r') {
            rest.pop();
        }
        if rest.is_empty() {
            return None;
        }
        Some(decode(&rest))
    }
}

/// Position de fin de la première ligne et début de la suivante ; `\r` final attend l'octet suivant.
fn find_line_end(bytes: &[u8]) -> Option<(usize, usize)> {
    let end = bytes.iter().position(|b| matches!(b, b'\n' | b'\r'))?;
    match (bytes[end], bytes.get(end + 1)) {
        (b'\n', _) => Some((end, end + 1)),
        (_, Some(b'\n')) => Some((end, end + 2)),
        (_, Some(_)) => Some((end, end + 1)),
        (_, None) => None,
    }
}

/// Décode une ligne complète ; un octet invalide devient le caractère de remplacement.
fn decode(line: &[u8]) -> String {
    String::from_utf8_lossy(line).into_owned()
}
