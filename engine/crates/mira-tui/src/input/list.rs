//! Touches d'action sur un message, valables dans la liste comme dans le lecteur.

use crossterm::event::{KeyCode, KeyEvent};

use super::keys::lookup;
use crate::update::{ComposeIntent, Intent};

pub const MESSAGE_KEYS: [(KeyCode, Intent); 11] = [
    (KeyCode::Char('e'), Intent::Archive),
    (KeyCode::Char('d'), Intent::Trash),
    (KeyCode::Char('D'), Intent::DeletePermanently),
    (KeyCode::Char('s'), Intent::ToggleFlag),
    (KeyCode::Char('u'), Intent::ToggleRead),
    (KeyCode::Char('m'), Intent::OpenMovePicker),
    (KeyCode::Char('z'), Intent::Undo),
    (KeyCode::Char('r'), Intent::Compose(ComposeIntent::Reply)),
    (KeyCode::Char('a'), Intent::Compose(ComposeIntent::ReplyAll)),
    (KeyCode::Char('f'), Intent::Compose(ComposeIntent::Forward)),
    (KeyCode::Char('c'), Intent::Compose(ComposeIntent::New)),
];

/// Intention d'une touche de la liste de messages.
pub fn intent(key: KeyEvent) -> Option<Intent> {
    lookup(&MESSAGE_KEYS, key)
}
