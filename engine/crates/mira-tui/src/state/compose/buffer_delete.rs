//! Suppression dans le tampon : avant ou après le curseur, en recollant les lignes au besoin.

use super::char_offset::{byte_offset, char_len};
use super::text_buffer::TextBuffer;

impl TextBuffer {
    /// Efface le caractère avant le curseur, ou recolle la ligne à la précédente ; sans effet tout au début.
    pub fn backspace(&mut self) {
        if self.col > 0 {
            self.remove_at(self.col - 1);
            self.col -= 1;
            self.wanted = self.col;
        } else if self.line > 0 {
            self.join_with_previous();
        }
    }

    /// Efface le caractère sous le curseur, ou recolle la ligne suivante ; sans effet tout à la fin.
    pub fn delete(&mut self) {
        let Some(line) = self.lines.get(self.line) else {
            return;
        };
        if self.col < char_len(line) {
            self.remove_at(self.col);
        } else if self.line + 1 < self.lines.len() {
            let next = self.lines.remove(self.line + 1);
            if let Some(line) = self.lines.get_mut(self.line) {
                line.push_str(&next);
            }
        }
    }

    /// Retire le caractère d'indice `col` de la ligne courante, s'il existe.
    fn remove_at(&mut self, col: usize) {
        if let Some(line) = self.lines.get_mut(self.line) {
            let at = byte_offset(line, col);
            if at < line.len() {
                line.remove(at);
            }
        }
    }

    /// Recolle la ligne courante à la précédente ; le curseur se place à la jointure.
    fn join_with_previous(&mut self) {
        let current = self.lines.remove(self.line);
        self.line -= 1;
        if let Some(previous) = self.lines.get_mut(self.line) {
            self.col = char_len(previous);
            previous.push_str(&current);
        }
        self.wanted = self.col;
    }
}
