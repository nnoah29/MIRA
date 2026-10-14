//! Insertion dans le tampon : un caractère ou un saut de ligne au curseur.

use super::char_offset::byte_offset;
use super::text_buffer::TextBuffer;

impl TextBuffer {
    /// Insère un caractère au curseur ; un caractère de contrôle est refusé.
    pub fn insert_char(&mut self, c: char) {
        if c.is_control() {
            return;
        }
        let Some(line) = self.lines.get_mut(self.line) else {
            return;
        };
        line.insert(byte_offset(line, self.col), c);
        self.col += 1;
        self.wanted = self.col;
    }

    /// Coupe la ligne au curseur ; le curseur passe au début de la nouvelle ligne.
    pub fn insert_newline(&mut self) {
        let Some(line) = self.lines.get_mut(self.line) else {
            return;
        };
        let rest = line.split_off(byte_offset(line, self.col));
        self.lines.insert(self.line + 1, rest);
        self.line += 1;
        self.col = 0;
        self.wanted = 0;
    }
}
