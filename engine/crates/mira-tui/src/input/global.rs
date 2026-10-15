//! Touches communes aux trois panneaux : navigation, panneaux, aide, recherche, synchro, sortie.

use crossterm::event::{KeyCode, KeyEvent};

use super::context::{KeyContext, pane_context};
use super::keys::{is_ctrl, lookup};
use crate::state::AppState;
use crate::update::{Intent, PaletteIntent, SearchIntent};

const GLOBAL_KEYS: [(KeyCode, Intent); 16] = [
    (KeyCode::Tab, Intent::NextPane),
    (KeyCode::BackTab, Intent::PrevPane),
    (KeyCode::Char('?'), Intent::Help),
    (KeyCode::Char('/'), Intent::Search(SearchIntent::Open)),
    (KeyCode::Char('q'), Intent::Quit),
    (KeyCode::Char('j'), Intent::MoveDown),
    (KeyCode::Down, Intent::MoveDown),
    (KeyCode::Char('k'), Intent::MoveUp),
    (KeyCode::Up, Intent::MoveUp),
    (KeyCode::Char('g'), Intent::Top),
    (KeyCode::Home, Intent::Top),
    (KeyCode::Char('G'), Intent::Bottom),
    (KeyCode::End, Intent::Bottom),
    (KeyCode::Enter, Intent::Open),
    (KeyCode::Esc, Intent::Back),
    (KeyCode::Char(':'), Intent::Palette(PaletteIntent::Open)),
];

/// Intention d'une touche commune ; Échap quitte d'abord les résultats de recherche de la liste.
pub fn intent(state: &AppState, key: KeyEvent) -> Option<Intent> {
    if is_ctrl(key, 'r') {
        return Some(Intent::StartSync);
    }
    let intent = lookup(&GLOBAL_KEYS, key)?;
    if intent == Intent::Back && pane_context(state) == KeyContext::SearchResults {
        return Some(Intent::Search(SearchIntent::Leave));
    }
    Some(intent)
}
