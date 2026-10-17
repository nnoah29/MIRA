//! Touches du lecteur : les actions sur le message ouvert, comme dans la liste.

use crossterm::event::KeyEvent;

use super::keys::lookup;
use super::list::MESSAGE_KEYS;
use crate::update::Intent;

/// Intention d'une touche du lecteur ; le message ouvert est celui sélectionné dans la liste.
pub fn intent(key: KeyEvent) -> Option<Intent> {
    lookup(&MESSAGE_KEYS, key)
}
