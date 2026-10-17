//! Tampon d'édition de plusieurs lignes ; le curseur se compte en caractères, jamais en octets.

use super::char_offset::char_len;

/// Lignes (sans saut de ligne), curseur (ligne, colonne), colonne souhaitée et première ligne affichée.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextBuffer {
    pub(super) lines: Vec<String>,
    pub(super) line: usize,
    pub(super) col: usize,
    pub(super) wanted: usize,
    pub(super) top: usize,
}

impl TextBuffer {
    /// Tampon de plusieurs lignes, curseur au début ; `\r\n` et `\r` deviennent `\n`.
    pub fn new(text: &str) -> TextBuffer {
        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
        TextBuffer {
            lines: normalized.split('\n').map(str::to_string).collect(),
            line: 0,
            col: 0,
            wanted: 0,
            top: 0,
        }
    }

    /// Tampon d'une seule ligne, curseur à la fin ; sauts de ligne et tabulations deviennent des espaces.
    pub fn single_line(text: &str) -> TextBuffer {
        let flat: String = text
            .chars()
            .map(|c| if c.is_whitespace() { ' ' } else { c })
            .filter(|c| !c.is_control())
            .collect();
        let end = char_len(&flat);
        TextBuffer {
            lines: vec![flat],
            line: 0,
            col: end,
            wanted: end,
            top: 0,
        }
    }

    /// Texte complet, lignes jointes par `\n`.
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    /// Lignes du tampon, sans saut de ligne.
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// Position du curseur : (ligne, colonne en caractères).
    pub fn cursor(&self) -> (usize, usize) {
        (self.line, self.col)
    }
}
