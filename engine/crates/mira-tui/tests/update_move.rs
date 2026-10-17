//! Sélecteur de déplacement : boîtes proposées, choix et annulation.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use action_fixtures::{action_command, state_with_mailboxes};
use mira_assembly::mira_vocabulary::{ActionKind, MailboxId};
use mira_tui::state::Popup;
use mira_tui::update::Intent;
use navigation_fixtures::press;

/// Le sélecteur propose les autres boîtes du même compte, pas la boîte courante ni un autre compte.
#[test]
fn move_picker_lists_other_mailboxes_of_same_account() {
    let mut state = state_with_mailboxes();
    assert!(press(&mut state, Intent::OpenMovePicker).is_empty());
    let Popup::MovePicker(picker) = &state.popup else {
        panic!("sélecteur attendu : {:?}", state.popup);
    };
    let names: Vec<&str> = picker.targets().iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, vec!["Archive", "Projets"]);
}

/// Choisir une boîte envoie `Move` vers elle et retire le message de la liste.
#[test]
fn move_picker_choice_sends_move() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::OpenMovePicker);
    press(&mut state, Intent::MoveDown);
    let commands = press(&mut state, Intent::Confirm);
    let kind = ActionKind::Move {
        to: MailboxId::from_raw("a1-Projets"),
    };
    assert_eq!(commands, vec![action_command(kind, &["m1"], "tui-1-act-1")]);
    assert_eq!(state.popup, Popup::Closed);
    assert_eq!(state.messages.len(), 1);
}

/// Annuler le sélecteur n'envoie rien.
#[test]
fn move_picker_cancel_sends_nothing() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::OpenMovePicker);
    assert!(press(&mut state, Intent::Cancel).is_empty());
    assert_eq!(state.popup, Popup::Closed);
    assert_eq!(state.messages.len(), 2);
}

/// Sans autre boîte, le sélecteur ne s'ouvre pas et le dit.
#[test]
fn move_picker_without_target_shows_status() {
    let mut state = state_with_mailboxes();
    state.mailboxes.replace_items(Vec::new());
    press(&mut state, Intent::OpenMovePicker);
    assert_eq!(state.popup, Popup::Closed);
    assert!(state.status.message.is_some());
}

/// Fenêtre ouverte, les autres intentions ne touchent ni la liste ni les panneaux.
#[test]
fn modal_swallows_other_intents() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::DeletePermanently);
    assert!(press(&mut state, Intent::Archive).is_empty());
    assert!(press(&mut state, Intent::NextPane).is_empty());
    assert_eq!(state.messages.len(), 2);
    assert_eq!(state.focus, mira_tui::state::Pane::List);
}
