//! Neutralisation de tout texte venant du courrier ou du serveur avant affichage.

use super::ansi_skip::{skip_csi, skip_escape, skip_string};

/// Texte sur une ligne : séquences et caractères de contrôle retirés, sauts de ligne en espaces.
pub fn display_safe(text: &str) -> String {
    sanitize(text, false)
}

/// Texte sur plusieurs lignes : comme `display_safe` mais les sauts de ligne sont conservés.
pub fn display_safe_multiline(text: &str) -> String {
    sanitize(text, true)
}

/// Parcourt le texte en retirant ce qui pourrait piloter le terminal.
fn sanitize(text: &str, keep_newlines: bool) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\u{1b}' => skip_escape(&mut chars),
            '\u{9b}' => skip_csi(&mut chars),
            '\u{90}' | '\u{98}' | '\u{9d}' | '\u{9e}' | '\u{9f}' => skip_string(&mut chars),
            '\n' if keep_newlines => out.push('\n'),
            '\n' | '\t' => out.push(' '),
            c if c.is_control() || is_direction_control(c) => {}
            c => out.push(c),
        }
    }
    out
}

/// Vrai pour les marques de direction du texte, qui peuvent inverser l'affichage.
fn is_direction_control(c: char) -> bool {
    matches!(c, '\u{200e}' | '\u{200f}' | '\u{061c}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}
