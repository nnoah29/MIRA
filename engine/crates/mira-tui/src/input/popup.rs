//! Touches des fenêtres modales : seules les leurs comptent.

use crossterm::event::{KeyCode, KeyEvent};

use super::keys::lookup;
use crate::update::{ComposeIntent, Intent};

const CONFIRM_KEYS: [(KeyCode, Intent); 4] = [
    (KeyCode::Enter, Intent::Confirm),
    (KeyCode::Char('y'), Intent::Confirm),
    (KeyCode::Esc, Intent::Cancel),
    (KeyCode::Char('n'), Intent::Cancel),
];

const PICKER_KEYS: [(KeyCode, Intent); 6] = [
    (KeyCode::Char('j'), Intent::MoveDown),
    (KeyCode::Down, Intent::MoveDown),
    (KeyCode::Char('k'), Intent::MoveUp),
    (KeyCode::Up, Intent::MoveUp),
    (KeyCode::Enter, Intent::Confirm),
    (KeyCode::Esc, Intent::Cancel),
];

const CLOSE_DRAFT_KEYS: [(KeyCode, Intent); 5] = [
    (
        KeyCode::Char('s'),
        Intent::Compose(ComposeIntent::SaveAndClose),
    ),
    (KeyCode::Char('d'), Intent::Compose(ComposeIntent::Discard)),
    (KeyCode::Enter, Intent::Confirm),
    (KeyCode::Char('c'), Intent::Cancel),
    (KeyCode::Esc, Intent::Cancel),
];

const HELP_KEYS: [(KeyCode, Intent); 4] = [
    (KeyCode::Esc, Intent::Cancel),
    (KeyCode::Char('?'), Intent::Cancel),
    (KeyCode::Char('q'), Intent::Cancel),
    (KeyCode::Enter, Intent::Cancel),
];

/// Touches de la confirmation de suppression.
pub fn confirm_intent(key: KeyEvent) -> Option<Intent> {
    lookup(&CONFIRM_KEYS, key)
}

/// Touches du sélecteur de déplacement.
pub fn picker_intent(key: KeyEvent) -> Option<Intent> {
    lookup(&PICKER_KEYS, key)
}

/// Touches de la confirmation de fermeture d'un brouillon modifié.
pub fn close_draft_intent(key: KeyEvent) -> Option<Intent> {
    lookup(&CLOSE_DRAFT_KEYS, key)
}

/// Touches de la fenêtre d'aide : toutes la ferment.
pub fn help_intent(key: KeyEvent) -> Option<Intent> {
    lookup(&HELP_KEYS, key)
}
