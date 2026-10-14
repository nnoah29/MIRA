//! Touches pendant la saisie de recherche : tout caractère est du texte.

use crossterm::event::{KeyCode, KeyEvent};

use super::keys::{is_plain, lookup};
use crate::update::{Intent, SearchIntent};

const SEARCH_KEYS: [(KeyCode, Intent); 3] = [
    (KeyCode::Backspace, Intent::Search(SearchIntent::Backspace)),
    (KeyCode::Enter, Intent::Search(SearchIntent::Submit)),
    (KeyCode::Esc, Intent::Search(SearchIntent::Leave)),
];

/// Intention d'une touche dans la saisie : un caractère s'ajoute, même `q`, `j` ou `/`.
pub fn intent(key: KeyEvent) -> Option<Intent> {
    if let KeyCode::Char(c) = key.code {
        return is_plain(key).then_some(Intent::Search(SearchIntent::Char(c)));
    }
    lookup(&SEARCH_KEYS, key)
}
