//! Touches propres à la barre latérale : changement de compte.

use crossterm::event::{KeyCode, KeyEvent};

use super::keys::lookup;
use crate::update::Intent;

const SIDEBAR_KEYS: [(KeyCode, Intent); 2] = [
    (KeyCode::Char(']'), Intent::NextAccount),
    (KeyCode::Char('['), Intent::PrevAccount),
];

/// Intention d'une touche de la barre latérale.
pub fn intent(key: KeyEvent) -> Option<Intent> {
    lookup(&SIDEBAR_KEYS, key)
}
