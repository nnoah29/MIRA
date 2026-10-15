//! Touches de la palette de commandes : tout caractère est du texte.

use crossterm::event::{KeyCode, KeyEvent};

use super::keys::{is_typed, lookup};
use crate::update::{Intent, PaletteIntent};

const PALETTE_KEYS: [(KeyCode, Intent); 3] = [
    (
        KeyCode::Backspace,
        Intent::Palette(PaletteIntent::Backspace),
    ),
    (KeyCode::Enter, Intent::Palette(PaletteIntent::Submit)),
    (KeyCode::Esc, Intent::Palette(PaletteIntent::Close)),
];

/// Intention d'une touche dans la palette : un caractère s'ajoute, même `q`, `j` ou `:`.
pub fn intent(key: KeyEvent) -> Option<Intent> {
    if let KeyCode::Char(c) = key.code {
        return is_typed(key).then_some(Intent::Palette(PaletteIntent::Char(c)));
    }
    lookup(&PALETTE_KEYS, key)
}
