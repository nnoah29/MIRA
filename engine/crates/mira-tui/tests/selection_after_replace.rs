//! La sélection reste dans les bornes quand la liste est remplacée.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod navigation_fixtures;

use mira_tui::state::AppState;
use mira_tui::update::Intent;
use navigation_fixtures::{numbered, press, state_with_messages};

/// Liste vide dès le départ : tout déplacement laisse la sélection absente.
#[test]
fn empty_list_selection_stays_none() {
    let mut state = state_with_messages(0);
    for intent in [
        Intent::MoveDown,
        Intent::MoveUp,
        Intent::Bottom,
        Intent::Top,
    ] {
        press(&mut state, intent);
    }
    assert_eq!(state.messages.selection.selected, None);
}

/// Une liste vidée efface la sélection et les déplacements la laissent absente.
#[test]
fn emptied_list_clears_selection() {
    let mut state = state_with_messages(5);
    press(&mut state, Intent::Bottom);
    state.messages.replace_items(Vec::new());
    assert_eq!(state.messages.selection.selected, None);
    press(&mut state, Intent::MoveDown);
    assert_eq!(state.messages.selection.selected, None);
}

/// Une liste raccourcie ramène la sélection sur le dernier élément.
#[test]
fn shortened_list_clamps_selection() {
    let mut state = state_with_messages(10);
    state.messages.selection.selected = Some(7);
    state.messages.replace_items(numbered(2));
    assert_eq!(state.messages.selection.selected, Some(1));
}

/// Une liste vide puis remplie sélectionne le premier élément.
#[test]
fn filled_list_selects_first() {
    let mut state = AppState::new(80, 24);
    state.mailboxes.replace_items(numbered(3));
    assert_eq!(state.mailboxes.selection.selected, Some(0));
    state.accounts.replace_items(numbered(1));
    assert_eq!(state.accounts.selection.selected, Some(0));
}
