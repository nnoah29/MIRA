//! Champs d'adresses (À, Cc, Cci) : une ligne ; Entrée et bas passent au champ suivant, haut au précédent.

use super::intent::EditKey;
use super::text_keys;
use crate::state::compose::ComposeState;

/// Applique une touche à un champ d'adresses.
pub fn apply(compose: &mut ComposeState, key: EditKey) {
    match key {
        EditKey::Newline | EditKey::Down => compose.focus_next(),
        EditKey::Up => compose.focus_above(),
        EditKey::Char(_)
        | EditKey::Backspace
        | EditKey::Delete
        | EditKey::Left
        | EditKey::Right
        | EditKey::Home
        | EditKey::End => text_keys::apply(compose, key),
    }
}
