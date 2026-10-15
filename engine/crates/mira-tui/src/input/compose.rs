//! Touches de l'éditeur de brouillon : toute touche imprimable est du texte, jamais une action globale.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::keys::{is_ctrl, is_plain, lookup};
use crate::update::{ComposeIntent, EditKey, Intent};

const CREATING_KEYS: [(KeyCode, Intent); 1] =
    [(KeyCode::Esc, Intent::Compose(ComposeIntent::Close))];

const COMPOSE_KEYS: [(KeyCode, Intent); 12] = [
    (KeyCode::Esc, Intent::Compose(ComposeIntent::Close)),
    (KeyCode::Tab, Intent::Compose(ComposeIntent::NextField)),
    (KeyCode::BackTab, Intent::Compose(ComposeIntent::PrevField)),
    (
        KeyCode::Enter,
        Intent::Compose(ComposeIntent::Edit(EditKey::Newline)),
    ),
    (
        KeyCode::Backspace,
        Intent::Compose(ComposeIntent::Edit(EditKey::Backspace)),
    ),
    (
        KeyCode::Delete,
        Intent::Compose(ComposeIntent::Edit(EditKey::Delete)),
    ),
    (
        KeyCode::Left,
        Intent::Compose(ComposeIntent::Edit(EditKey::Left)),
    ),
    (
        KeyCode::Right,
        Intent::Compose(ComposeIntent::Edit(EditKey::Right)),
    ),
    (
        KeyCode::Up,
        Intent::Compose(ComposeIntent::Edit(EditKey::Up)),
    ),
    (
        KeyCode::Down,
        Intent::Compose(ComposeIntent::Edit(EditKey::Down)),
    ),
    (
        KeyCode::Home,
        Intent::Compose(ComposeIntent::Edit(EditKey::Home)),
    ),
    (
        KeyCode::End,
        Intent::Compose(ComposeIntent::Edit(EditKey::End)),
    ),
];

/// Pendant la création du brouillon seul Échap compte (il annule) ; tout le reste est ignoré, jamais mis en file.
pub fn creating_intent(key: KeyEvent) -> Option<Intent> {
    lookup(&CREATING_KEYS, key)
}

/// Vrai pour Ctrl+Alt ensemble, que certains terminaux envoient pour AltGr.
fn is_alt_gr(key: KeyEvent) -> bool {
    key.modifiers
        .contains(KeyModifiers::CONTROL | KeyModifiers::ALT)
}

/// Intention d'une touche dans l'éditeur : Ctrl+Entrée envoie, Ctrl+s enregistre, le reste est du texte.
pub fn intent(key: KeyEvent) -> Option<Intent> {
    if is_send_key(key) {
        return Some(Intent::Compose(ComposeIntent::Send));
    }
    if is_ctrl(key, 's') && !is_alt_gr(key) {
        return Some(Intent::Compose(ComposeIntent::Save));
    }
    if let KeyCode::Char(c) = key.code {
        let typed = is_plain(key) || is_alt_gr(key);
        return typed.then_some(Intent::Compose(ComposeIntent::Edit(EditKey::Char(c))));
    }
    if key.code == KeyCode::Tab && key.modifiers.contains(KeyModifiers::SHIFT) {
        return Some(Intent::Compose(ComposeIntent::PrevField));
    }
    lookup(&COMPOSE_KEYS, key)
}

/// Vrai pour Ctrl+Entrée, ou Ctrl+j que des terminaux envoient à sa place (même code que le saut de ligne).
fn is_send_key(key: KeyEvent) -> bool {
    let ctrl_enter = key.code == KeyCode::Enter && key.modifiers.contains(KeyModifiers::CONTROL);
    ctrl_enter || (is_ctrl(key, 'j') && !is_alt_gr(key))
}
