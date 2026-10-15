//! Seconde confirmation d'abandon : seul un `D` immédiat abandonne ; toute autre touche ramène à l'éditeur sans rien détruire.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod input_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use compose_fixtures::{delete_command, open_modified, screen};
use crossterm::event::{KeyCode, KeyEvent};
use input_fixtures::{code, key, type_keys};
use mira_tui::state::compose::Field;
use mira_tui::state::{AppState, Popup};

/// Échap par réflexe, puis ce texte tapé touche par touche ; rend les commandes produites.
fn escape_then_type(state: &mut AppState, text: &str) -> Vec<mira_tui::commands::Command> {
    let mut keys: Vec<KeyEvent> = vec![code(KeyCode::Esc)];
    keys.extend(text.chars().map(key));
    type_keys(state, &keys)
}

/// Rien n'a été détruit : éditeur ouvert, objet modifié conservé (la suite de la frappe s'y ajoute), aucune suppression.
fn assert_nothing_destroyed(state: &AppState, commands: &[mira_tui::commands::Command]) {
    assert!(!commands.contains(&delete_command("d1")), "{commands:?}");
    let open = state.compose.open().unwrap();
    let subject = open.text(Field::Subject);
    assert!(subject.starts_with("Re: Bonjour!"), "{subject:?}");
    assert_ne!(state.popup, Popup::ConfirmDiscardDraft);
}

/// « Dear Dave » et « Dupont et Durand » tapés après Échap n'abandonnent rien.
#[test]
fn typing_words_with_capital_d_never_discards() {
    for text in ["Dear Dave", "Dupont et Durand"] {
        let mut state = open_modified();
        let commands = escape_then_type(&mut state, text);
        assert_nothing_destroyed(&state, &commands);
    }
}

/// Verr. Maj : « DAVE » n'abandonne rien ; `D` puis `x` revient à l'éditeur.
#[test]
fn any_other_key_returns_to_the_editor() {
    let mut state = open_modified();
    let commands = escape_then_type(&mut state, "DAVE");
    assert_nothing_destroyed(&state, &commands);
    let mut state = open_modified();
    let commands = escape_then_type(&mut state, "Dx");
    assert_nothing_destroyed(&state, &commands);
    assert_eq!(state.popup, Popup::Closed);
}

/// `D` puis `D` immédiat abandonne ; la seconde fenêtre dit que toute autre touche ramène à l'éditeur.
#[test]
fn immediate_second_d_discards() {
    let mut state = open_modified();
    escape_then_type(&mut state, "D");
    let text = screen(&state, 100, 30).0;
    assert!(text.contains("toute autre touche"), "{text}");
    assert_eq!(
        type_keys(&mut state, &[key('D')]),
        vec![delete_command("d1")]
    );
    assert!(state.compose.is_closed());
}
