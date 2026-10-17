//! Touches de la vue « Brouillons » et de sa confirmation de suppression : seules les leurs comptent.

use crossterm::event::{KeyCode, KeyEvent};

use super::keys::{is_plain, lookup};
use crate::update::{DraftsIntent, Intent};

const DRAFTS_KEYS: [(KeyCode, Intent); 10] = [
    (KeyCode::Char('j'), Intent::MoveDown),
    (KeyCode::Down, Intent::MoveDown),
    (KeyCode::Char('k'), Intent::MoveUp),
    (KeyCode::Up, Intent::MoveUp),
    (KeyCode::Char('g'), Intent::Top),
    (KeyCode::Char('G'), Intent::Bottom),
    (KeyCode::Enter, Intent::Open),
    (KeyCode::Char('D'), Intent::Drafts(DraftsIntent::AskDelete)),
    (KeyCode::Esc, Intent::Back),
    (KeyCode::Char('q'), Intent::Quit),
];

/// Touches de la vue « Brouillons ».
pub fn intent(key: KeyEvent) -> Option<Intent> {
    lookup(&DRAFTS_KEYS, key)
}

/// Confirmation de suppression : seul un `D` immédiat (sans Ctrl ni Alt) supprime ; toute autre touche annule.
pub fn confirm_delete_intent(key: KeyEvent) -> Option<Intent> {
    let capital_d = key.code == KeyCode::Char('D') && is_plain(key);
    Some(if capital_d {
        Intent::Drafts(DraftsIntent::ConfirmDelete)
    } else {
        Intent::Cancel
    })
}
