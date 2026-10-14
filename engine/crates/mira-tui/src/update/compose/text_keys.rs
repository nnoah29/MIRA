//! Touches de texte communes à tous les champs : caractères, effacement, déplacement horizontal.

use super::intent::EditKey;
use crate::state::compose::{ComposeState, TextBuffer};

/// Applique une touche de texte ; Entrée, haut et bas dépendent du champ et ne font rien ici.
pub fn apply(compose: &mut ComposeState, key: EditKey) {
    match key {
        EditKey::Char(c) => compose.edit_text(|buffer| buffer.insert_char(c)),
        EditKey::Backspace => compose.edit_text(TextBuffer::backspace),
        EditKey::Delete => compose.edit_text(TextBuffer::delete),
        EditKey::Left => compose.move_cursor(TextBuffer::left),
        EditKey::Right => compose.move_cursor(TextBuffer::right),
        EditKey::Home => compose.move_cursor(TextBuffer::home),
        EditKey::End => compose.move_cursor(TextBuffer::end),
        EditKey::Newline | EditKey::Up | EditKey::Down => {}
    }
}
