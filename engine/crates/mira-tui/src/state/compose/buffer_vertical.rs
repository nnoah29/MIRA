//! Déplacements verticaux du curseur : la colonne souhaitée survit aux lignes courtes.

use super::text_buffer::TextBuffer;

impl TextBuffer {
    /// Ligne du dessus à la colonne souhaitée (ou à sa fin) ; sans effet sur la première.
    pub fn up(&mut self) {
        if self.line > 0 {
            self.line -= 1;
            self.col = self.wanted.min(self.line_len(self.line));
        }
    }

    /// Ligne du dessous à la colonne souhaitée (ou à sa fin) ; sans effet sur la dernière.
    pub fn down(&mut self) {
        if self.line + 1 < self.lines.len() {
            self.line += 1;
            self.col = self.wanted.min(self.line_len(self.line));
        }
    }
}
