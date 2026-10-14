//! Lectures du tampon utiles à l'affichage et à la comparaison.

use super::char_offset::char_len;
use super::text_buffer::TextBuffer;
use super::viewport::visible_top;

impl TextBuffer {
    /// Nombre de lignes.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Nombre de caractères de la ligne `index` (0 si elle n'existe pas).
    pub fn line_len(&self, index: usize) -> usize {
        self.lines.get(index).map_or(0, |line| char_len(line))
    }

    /// Première ligne affichée.
    pub fn top(&self) -> usize {
        self.top
    }

    /// Vrai si le contenu est exactement ce texte, sans construire le texte du tampon.
    pub fn matches(&self, text: &str) -> bool {
        self.lines.iter().map(String::as_str).eq(text.split('\n'))
    }

    /// Fait suivre la fenêtre au curseur pour `rows` lignes visibles.
    pub fn follow(&mut self, rows: usize) {
        self.top = visible_top(self.top, self.line, rows);
    }
}
