//! Curseur d'octets utilisé par l'analyseur.

/// Curseur de lecture sur une commande brute.
#[derive(Debug)]
pub(crate) struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    /// Crée un curseur au début des données
    pub(crate) fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// Octet courant sans avancer
    pub(crate) fn peek(&self) -> Option<u8> {
        self.data.get(self.pos).copied()
    }

    /// Octet courant, puis avance d'une position
    pub(crate) fn bump(&mut self) -> Option<u8> {
        let byte = self.peek();
        self.pos += usize::from(byte.is_some());
        byte
    }

    /// Ignore espaces, tabulations et fins de ligne
    pub(crate) fn skip_blanks(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\r' | b'\n')) {
            self.pos += 1;
        }
    }

    /// Passe un éventuel saut de ligne CRLF ou LF
    pub(crate) fn skip_line_end(&mut self) {
        if self.peek() == Some(b'\r') {
            self.pos += 1;
        }
        if self.peek() == Some(b'\n') {
            self.pos += 1;
        }
    }

    /// Prend exactement `count` octets s'ils sont disponibles
    pub(crate) fn take(&mut self, count: usize) -> Option<&'a [u8]> {
        let end = self.pos.checked_add(count)?;
        let slice = self.data.get(self.pos..end)?;
        self.pos = end;
        Some(slice)
    }
}
