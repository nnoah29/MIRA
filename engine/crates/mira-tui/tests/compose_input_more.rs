//! Clavier de la rédaction : Ctrl+c, relâchements et confirmation de fermeture.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod input_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod search_fixtures;
mod sidebar_fixtures;

use compose_fixtures::open_reply;
use crossterm::event::{KeyCode, KeyEventKind};
use input_fixtures::{code, ctrl, key, map, released};
use mira_tui::input::{KeyContext, context_of};
use mira_tui::update::{ComposeIntent, Intent};

/// Ctrl+c quitte même pendant l'édition ; un relâchement de touche ne tape rien.
#[test]
fn ctrl_c_quits_and_releases_are_ignored_while_editing() {
    assert_eq!(map(&open_reply(), ctrl('c')), Some(Intent::Quit));
    assert_eq!(map(&open_reply(), released('a')), None);
    let mut repeat = key('a');
    repeat.kind = KeyEventKind::Repeat;
    assert!(map(&open_reply(), repeat).is_some());
}

/// La confirmation de fermeture : `s` ou Entrée enregistrent, `D` demande l'abandon (`d` ne fait rien), `c` ou Échap continuent.
#[test]
fn close_confirmation_keys() {
    let mut state = open_reply();
    state.popup = mira_tui::state::Popup::ConfirmCloseDraft;
    assert_eq!(context_of(&state), KeyContext::ConfirmClose);
    assert_eq!(
        map(&state, key('s')),
        Some(Intent::Compose(ComposeIntent::SaveAndClose))
    );
    assert_eq!(map(&state, key('d')), None);
    assert_eq!(
        map(&state, key('D')),
        Some(Intent::Compose(ComposeIntent::AskDiscard))
    );
    assert_eq!(map(&state, code(KeyCode::Enter)), Some(Intent::Confirm));
    assert_eq!(map(&state, code(KeyCode::Esc)), Some(Intent::Cancel));
    assert_eq!(map(&state, key('c')), Some(Intent::Cancel));
    assert_eq!(map(&state, key('x')), None);
}
