//! Fabriques de touches clavier et traduction en intention pour les tests du clavier.
#![allow(dead_code)]

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use mira_tui::input::intent_for;
use mira_tui::state::AppState;
use mira_tui::update::Intent;

/// Touche spéciale ou caractère sans modificateur.
pub fn code(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// Caractère tapé sans modificateur ; une majuscule porte Maj comme le ferait le terminal.
pub fn key(c: char) -> KeyEvent {
    let modifiers = if c.is_uppercase() {
        KeyModifiers::SHIFT
    } else {
        KeyModifiers::NONE
    };
    KeyEvent::new(KeyCode::Char(c), modifiers)
}

/// Caractère tapé avec Ctrl.
pub fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

/// Relâchement d'une touche, que certains terminaux signalent.
pub fn released(c: char) -> KeyEvent {
    let mut event = key(c);
    event.kind = KeyEventKind::Release;
    event
}

/// Intention produite par cette touche dans cet état.
pub fn map(state: &AppState, event: KeyEvent) -> Option<Intent> {
    intent_for(state, event)
}
