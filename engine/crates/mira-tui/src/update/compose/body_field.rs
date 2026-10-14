//! Corps : plusieurs lignes ; Entrée coupe la ligne, haut et bas changent de ligne.

use super::intent::EditKey;
use super::text_keys;
use crate::state::compose::{ComposeState, TextBuffer};

/// Applique une touche au corps.
pub fn apply(compose: &mut ComposeState, key: EditKey) {
    match key {
        EditKey::Newline => compose.edit_text(TextBuffer::insert_newline),
        EditKey::Up => compose.move_cursor(TextBuffer::up),
        EditKey::Down => compose.move_cursor(TextBuffer::down),
        EditKey::Char(_)
        | EditKey::Backspace
        | EditKey::Delete
        | EditKey::Left
        | EditKey::Right
        | EditKey::Home
        | EditKey::End => text_keys::apply(compose, key),
    }
}
