//! Geste d'envoi délibéré : F5 ou Ctrl+Entrée réel ; Ctrl+j est un saut de ligne et n'ouvre jamais la confirmation.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod input_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod send_fixtures;
mod sidebar_fixtures;

use crossterm::event::{KeyCode, KeyEvent};
use input_fixtures::{code, ctrl, key, map};
use mira_tui::commands::Command;
use mira_tui::input::hints::hints_for;
use mira_tui::input::{KeyContext, context_of};
use mira_tui::state::compose::Field;
use mira_tui::state::{AppState, Popup};
use mira_tui::update::{ComposeIntent, EditKey, Intent};
use navigation_fixtures::press;

/// Fait passer ces touches par la table de touches puis par `update` ; rend toutes les commandes produites.
fn type_keys(state: &mut AppState, keys: &[KeyEvent]) -> Vec<Command> {
    let mut commands = Vec::new();
    for event in keys {
        if let Some(intent) = map(state, *event) {
            commands.extend(press(state, intent));
        }
    }
    commands
}

/// F5 demande l'envoi dans l'éditeur et ouvre la confirmation.
#[test]
fn f5_opens_the_send_confirmation() {
    let mut state = compose_fixtures::open_reply();
    let f5 = code(KeyCode::F(5));
    assert_eq!(map(&state, f5), Some(Intent::Compose(ComposeIntent::Send)));
    assert!(type_keys(&mut state, &[f5]).is_empty());
    assert_eq!(state.popup, Popup::ConfirmSend);
}

/// Ctrl+j insère un saut de ligne et n'ouvre rien.
#[test]
fn ctrl_j_is_a_newline_not_a_send() {
    let mut state = compose_fixtures::open_reply();
    let newline = Some(Intent::Compose(ComposeIntent::Edit(EditKey::Newline)));
    assert_eq!(map(&state, ctrl('j')), newline);
    type_keys(&mut state, &[ctrl('j')]);
    assert_eq!(state.popup, Popup::Closed);
    let body = state.compose.open().unwrap().text(Field::Body);
    assert_eq!(body, "\nCorps");
}

/// « Bonjour, » Ctrl+j « you » : le texte est « Bonjour,\nyou », aucun envoi, aucune confirmation.
#[test]
fn typing_y_after_ctrl_j_never_confirms_a_send() {
    let mut state = compose_fixtures::open_reply();
    let mut keys: Vec<KeyEvent> = "Bonjour,".chars().map(key).collect();
    keys.push(ctrl('j'));
    keys.extend("you".chars().map(key));
    let commands = type_keys(&mut state, &keys);
    assert_eq!(send_fixtures::count_sends(&commands), 0, "{commands:?}");
    assert_eq!(state.popup, Popup::Closed);
    let body = state.compose.open().unwrap().text(Field::Body);
    assert!(body.starts_with("Bonjour,\nyou"), "{body:?}");
}

/// Le pied de page et l'aide de l'éditeur montrent F5 pour envoyer.
#[test]
fn editor_hints_show_f5() {
    let state = compose_fixtures::open_reply();
    assert_eq!(context_of(&state), KeyContext::Compose);
    let shown = hints_for(KeyContext::Compose)
        .iter()
        .any(|hint| hint.keys.contains("F5") && hint.action == "envoyer");
    assert!(shown);
}
