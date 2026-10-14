//! Clavier de la rédaction : touches de démarrage hors saisie, tout est texte pendant l'édition.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod input_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod search_fixtures;
mod sidebar_fixtures;

use action_fixtures::state_with_mailboxes;
use compose_fixtures::open_reply;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use input_fixtures::{code, ctrl, key, map};
use mira_tui::input::{KeyContext, context_of};
use mira_tui::state::{AppState, Pane};
use mira_tui::update::{ComposeIntent, EditKey, Intent};

/// Intention de rédaction de cette touche dans l'éditeur ouvert.
fn composing(event: KeyEvent) -> Option<ComposeIntent> {
    match map(&open_reply(), event) {
        Some(Intent::Compose(intent)) => Some(intent),
        _ => None,
    }
}

/// `r`, `a`, `f` et `c` démarrent un brouillon dans la liste, les résultats et le lecteur, pas dans la barre latérale.
#[test]
fn start_keys_map_in_list_and_reader_only() {
    let table = [
        ('r', ComposeIntent::Reply),
        ('a', ComposeIntent::ReplyAll),
        ('f', ComposeIntent::Forward),
        ('c', ComposeIntent::New),
    ];
    for pane in [Pane::List, Pane::Reader] {
        let mut state: AppState = state_with_mailboxes();
        state.focus = pane;
        for (c, intent) in table {
            assert_eq!(
                map(&state, key(c)),
                Some(Intent::Compose(intent)),
                "{pane:?} {c}"
            );
        }
    }
    let mut sidebar = state_with_mailboxes();
    sidebar.focus = Pane::Sidebar;
    for (c, _) in table {
        assert_eq!(map(&sidebar, key(c)), None, "{c}");
    }
}

/// Pendant l'édition le contexte est celui de l'éditeur et toute touche imprimable est du texte.
#[test]
fn every_printable_key_is_text_while_editing() {
    let state = open_reply();
    assert_eq!(context_of(&state), KeyContext::Compose);
    for c in [
        'q', 'j', 'k', '/', '?', 'r', 'a', 'f', 'c', 'e', 'D', ' ', 'é', '日', '😀',
    ] {
        let typed = Some(Intent::Compose(ComposeIntent::Edit(EditKey::Char(c))));
        assert_eq!(map(&state, key(c)), typed, "{c}");
    }
}

/// Enregistrer, fermer, changer de champ, Entrée et déplacements ont leur sens, Ctrl+Entrée rien (envoi : tâche 11).
#[test]
fn special_keys_have_their_meaning() {
    let table = [
        (ctrl('s'), ComposeIntent::Save),
        (code(KeyCode::Esc), ComposeIntent::Close),
        (code(KeyCode::Tab), ComposeIntent::NextField),
        (code(KeyCode::BackTab), ComposeIntent::PrevField),
        (code(KeyCode::Enter), ComposeIntent::Edit(EditKey::Newline)),
        (
            code(KeyCode::Backspace),
            ComposeIntent::Edit(EditKey::Backspace),
        ),
        (code(KeyCode::Delete), ComposeIntent::Edit(EditKey::Delete)),
        (code(KeyCode::Left), ComposeIntent::Edit(EditKey::Left)),
        (code(KeyCode::Right), ComposeIntent::Edit(EditKey::Right)),
        (code(KeyCode::Up), ComposeIntent::Edit(EditKey::Up)),
        (code(KeyCode::Down), ComposeIntent::Edit(EditKey::Down)),
        (code(KeyCode::Home), ComposeIntent::Edit(EditKey::Home)),
        (code(KeyCode::End), ComposeIntent::Edit(EditKey::End)),
    ];
    for (event, intent) in table {
        assert_eq!(composing(event), Some(intent), "{event:?}");
    }
    let ctrl_enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL);
    assert_eq!(map(&open_reply(), ctrl_enter), None);
}

/// Ctrl ou Alt avec un caractère ne s'insèrent pas ; AltGr (Ctrl+Alt) insère ; Maj+Tab recule.
#[test]
fn modified_chars_are_not_text_except_altgr() {
    for modifiers in [KeyModifiers::CONTROL, KeyModifiers::ALT] {
        let event = KeyEvent::new(KeyCode::Char('x'), modifiers);
        assert_eq!(map(&open_reply(), event), None, "{modifiers:?}");
    }
    let altgr = KeyEvent::new(
        KeyCode::Char('€'),
        KeyModifiers::CONTROL | KeyModifiers::ALT,
    );
    let typed = Some(ComposeIntent::Edit(EditKey::Char('€')));
    assert_eq!(composing(altgr), typed);
    let shift_tab = KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT);
    assert_eq!(composing(shift_tab), Some(ComposeIntent::PrevField));
}
