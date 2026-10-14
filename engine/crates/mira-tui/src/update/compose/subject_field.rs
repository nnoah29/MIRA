//! Champ Objet : une ligne ; Entrée et bas passent au corps, haut aux adresses.

use super::intent::EditKey;
use super::text_keys;
use crate::state::compose::ComposeState;

/// Applique une touche à l'objet.
pub fn apply(compose: &mut ComposeState, key: EditKey) {
    match key {
        EditKey::Newline | EditKey::Down => compose.focus_next(),
        EditKey::Up => compose.focus_previous(),
        EditKey::Char(_)
        | EditKey::Backspace
        | EditKey::Delete
        | EditKey::Left
        | EditKey::Right
        | EditKey::Home
        | EditKey::End => text_keys::apply(compose, key),
    }
}
