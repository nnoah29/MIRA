//! Clavier sous une fenêtre modale (seules ses touches comptent) et touches sans effet.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod input_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use action_fixtures::state_with_mailboxes;
use crossterm::event::KeyCode;
use input_fixtures::{code, ctrl, key, map, released};
use mira_tui::state::Popup;
use mira_tui::update::{Intent, SearchIntent};
use navigation_fixtures::press;

/// La confirmation accepte Entrée et `y`, refuse Échap et `n`, ignore tout le reste.
#[test]
fn confirm_popup_only_accepts_its_keys() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::DeletePermanently);
    assert!(matches!(state.popup, Popup::ConfirmDelete { .. }));
    assert_eq!(map(&state, code(KeyCode::Enter)), Some(Intent::Confirm));
    assert_eq!(map(&state, key('y')), Some(Intent::Confirm));
    assert_eq!(map(&state, code(KeyCode::Esc)), Some(Intent::Cancel));
    assert_eq!(map(&state, key('n')), Some(Intent::Cancel));
    for c in ['e', 'd', 'q', 'j', '/', '?'] {
        assert_eq!(map(&state, key(c)), None, "{c}");
    }
    assert_eq!(map(&state, code(KeyCode::Tab)), None);
}

/// Le sélecteur de déplacement se parcourt avec j/k et les flèches, Entrée choisit, Échap annule.
#[test]
fn move_picker_keys_navigate_and_choose() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::OpenMovePicker);
    assert!(matches!(state.popup, Popup::MovePicker(_)));
    assert_eq!(map(&state, key('j')), Some(Intent::MoveDown));
    assert_eq!(map(&state, code(KeyCode::Down)), Some(Intent::MoveDown));
    assert_eq!(map(&state, key('k')), Some(Intent::MoveUp));
    assert_eq!(map(&state, code(KeyCode::Up)), Some(Intent::MoveUp));
    assert_eq!(map(&state, code(KeyCode::Enter)), Some(Intent::Confirm));
    assert_eq!(map(&state, code(KeyCode::Esc)), Some(Intent::Cancel));
    assert_eq!(map(&state, key('y')), None);
    assert_eq!(map(&state, key('e')), None);
}

/// L'aide se ferme par Échap, `?`, `q` ou Entrée, et ne laisse passer aucune autre touche.
#[test]
fn help_popup_closes_on_escape_question_mark_or_q() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Help);
    assert_eq!(state.popup, Popup::Help);
    for event in [code(KeyCode::Esc), key('?'), key('q'), code(KeyCode::Enter)] {
        assert_eq!(map(&state, event), Some(Intent::Cancel), "{event:?}");
    }
    assert_eq!(map(&state, key('e')), None);
    press(&mut state, Intent::Cancel);
    assert_eq!(state.popup, Popup::Closed);
}

/// Ctrl+c quitte partout, même pendant la saisie de recherche ou sous une fenêtre.
#[test]
fn ctrl_c_quits_even_while_typing_or_under_popup() {
    let mut state = state_with_mailboxes();
    assert_eq!(map(&state, ctrl('c')), Some(Intent::Quit));
    press(&mut state, Intent::Search(SearchIntent::Open));
    assert_eq!(map(&state, ctrl('c')), Some(Intent::Quit));
    press(&mut state, Intent::Search(SearchIntent::Leave));
    press(&mut state, Intent::DeletePermanently);
    assert_eq!(map(&state, ctrl('c')), Some(Intent::Quit));
}

/// Relâchements, touches inconnues et Ctrl+lettre non prévu ne donnent rien.
#[test]
fn releases_and_unknown_keys_map_to_nothing() {
    let state = state_with_mailboxes();
    assert_eq!(map(&state, released('j')), None);
    assert_eq!(map(&state, key('x')), None);
    assert_eq!(map(&state, ctrl('e')), None);
    assert_eq!(map(&state, code(KeyCode::F(5))), None);
}
