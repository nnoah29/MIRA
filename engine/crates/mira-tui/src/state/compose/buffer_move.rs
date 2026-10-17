//! Déplacements horizontaux du curseur, qui fixent la colonne souhaitée.

use super::text_buffer::TextBuffer;

impl TextBuffer {
    /// Un caractère à gauche, ou fin de la ligne précédente.
    pub fn left(&mut self) {
        if self.col > 0 {
            self.col -= 1;
        } else if self.line > 0 {
            self.line -= 1;
            self.col = self.line_len(self.line);
        }
        self.wanted = self.col;
    }

    /// Un caractère à droite, ou début de la ligne suivante.
    pub fn right(&mut self) {
        if self.col < self.line_len(self.line) {
            self.col += 1;
        } else if self.line + 1 < self.lines.len() {
            self.line += 1;
            self.col = 0;
        }
        self.wanted = self.col;
    }

    /// Début de la ligne.
    pub fn home(&mut self) {
        self.col = 0;
        self.wanted = 0;
    }

    /// Fin de la ligne.
    pub fn end(&mut self) {
        self.col = self.line_len(self.line);
        self.wanted = self.col;
    }
}
