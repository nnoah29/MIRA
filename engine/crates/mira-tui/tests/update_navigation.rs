//! Navigation pure : panneaux, sélection bornée, sortie et redimensionnement.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod navigation_fixtures;

use mira_tui::commands::Command;
use mira_tui::state::{AppState, Pane};
use mira_tui::update::{Intent, Msg, update};
use navigation_fixtures::{press, state_with_messages};

/// Tab et Maj+Tab font tourner le focus dans les deux sens.
#[test]
fn tab_cycles_panes_forward_and_back() {
    let mut state = AppState::new(80, 24);
    assert_eq!(state.focus, Pane::Sidebar);
    press(&mut state, Intent::NextPane);
    assert_eq!(state.focus, Pane::List);
    press(&mut state, Intent::NextPane);
    press(&mut state, Intent::NextPane);
    assert_eq!(state.focus, Pane::Sidebar);
    press(&mut state, Intent::PrevPane);
    assert_eq!(state.focus, Pane::Reader);
}

/// La descente s'arrête sur le dernier élément et Top/Bottom bornent la sélection.
#[test]
fn move_down_stops_at_last_item() {
    let mut state = state_with_messages(3);
    for _ in 0..10 {
        press(&mut state, Intent::MoveDown);
    }
    assert_eq!(state.messages.selection.selected, Some(2));
    press(&mut state, Intent::Top);
    assert_eq!(state.messages.selection.selected, Some(0));
    press(&mut state, Intent::Bottom);
    assert_eq!(state.messages.selection.selected, Some(2));
}

/// La montée s'arrête sur le premier élément.
#[test]
fn move_up_stops_at_first_item() {
    let mut state = state_with_messages(3);
    press(&mut state, Intent::MoveDown);
    for _ in 0..10 {
        press(&mut state, Intent::MoveUp);
    }
    assert_eq!(state.messages.selection.selected, Some(0));
}

/// Une liste vide garde une sélection absente quel que soit le déplacement.
#[test]
fn empty_list_selection_stays_none() {
    let mut state = state_with_messages(0);
    press(&mut state, Intent::MoveDown);
    press(&mut state, Intent::MoveUp);
    press(&mut state, Intent::Bottom);
    press(&mut state, Intent::Top);
    assert_eq!(state.messages.selection.selected, None);
}

/// Quitter lève l'indicateur de sortie et demande la commande Quit.
#[test]
fn quit_intent_sets_should_quit_and_returns_quit_command() {
    let mut state = AppState::new(80, 24);
    let commands = press(&mut state, Intent::Quit);
    assert!(state.should_quit);
    assert_eq!(commands, vec![Command::Quit]);
}

/// Le redimensionnement met à jour la taille sans effet de bord.
#[test]
fn resize_updates_screen_size() {
    let mut state = AppState::new(80, 24);
    let commands = update(&mut state, Msg::Resize(120, 40));
    assert_eq!((state.width, state.height), (120, 40));
    assert!(commands.is_empty());
}
