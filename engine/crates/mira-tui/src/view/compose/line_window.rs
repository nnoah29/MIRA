//! Fenêtre horizontale d'une ligne : seuls les caractères visibles autour du curseur sont construits.

use ratatui::text::Span;

use crate::state::compose::char_offset::byte_offset;

/// Largeur d'affichage d'un caractère en colonnes (0 pour un caractère de contrôle, 2 pour 日).
pub fn char_width(c: char) -> usize {
    let mut buffer = [0u8; 4];
    Span::raw(&*c.encode_utf8(&mut buffer)).width()
}

/// Premier caractère à montrer pour que le curseur tienne dans `width` colonnes, cellule du curseur comprise.
pub fn left_edge(line: &str, cursor: usize, width: usize) -> usize {
    let before = line.get(..byte_offset(line, cursor)).unwrap_or("");
    let mut used = 0;
    let mut start = cursor;
    for c in before.chars().rev() {
        let next = used + char_width(c);
        if next >= width {
            break;
        }
        used = next;
        start -= 1;
    }
    start
}

/// Caractères de la ligne à partir de `start` qui tiennent dans `width` colonnes.
pub fn visible_slice(line: &str, start: usize, width: usize) -> String {
    let mut used = 0;
    let mut shown = String::new();
    for c in line.chars().skip(start) {
        used += char_width(c);
        if used > width {
            break;
        }
        shown.push(c);
    }
    shown
}

/// Colonnes entre le premier caractère montré et le curseur.
pub fn cursor_offset(line: &str, start: usize, cursor: usize) -> usize {
    line.chars()
        .skip(start)
        .take(cursor.saturating_sub(start))
        .map(char_width)
        .sum()
}
