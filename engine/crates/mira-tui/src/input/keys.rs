//! Lecture d'une touche : modificateurs et recherche dans une table de correspondance.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::update::Intent;

/// Vrai pour Ctrl+`c` (majuscule ou minuscule).
pub fn is_ctrl(key: KeyEvent, c: char) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(key.code, KeyCode::Char(typed) if typed.eq_ignore_ascii_case(&c))
}

/// Vrai si la touche ne porte ni Ctrl ni Alt (Maj est permise : elle fait les majuscules).
pub fn is_plain(key: KeyEvent) -> bool {
    !key.modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
}

/// Vrai pour un caractère à taper : sans Ctrl ni Alt, ou avec Ctrl+Alt ensemble (AltGr de certains terminaux).
pub fn is_typed(key: KeyEvent) -> bool {
    is_plain(key) || key.modifiers.contains(KeyModifiers::CONTROL | KeyModifiers::ALT)
}

/// Intention associée à une touche sans Ctrl ni Alt dans cette table.
pub fn lookup(table: &[(KeyCode, Intent)], key: KeyEvent) -> Option<Intent> {
    if !is_plain(key) {
        return None;
    }
    table
        .iter()
        .find(|(code, _)| *code == key.code)
        .map(|(_, intent)| *intent)
}
