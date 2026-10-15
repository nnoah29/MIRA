//! Fermeture du formulaire d'ajout de compte : Échap, confirmation si des champs sont remplis, mot de passe effacé.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod account_form_fixtures;
mod input_fixtures;

use account_form_fixtures::{SECRET, hit, known_provider, ready_state, type_text};
use crossterm::event::KeyCode;
use input_fixtures::{key, type_keys};
use mira_tui::state::AppState;

/// Un formulaire vide se ferme tout de suite avec Échap.
#[test]
fn escape_closes_an_empty_form_at_once() {
    let mut state = AppState::new(100, 30, 1);
    type_keys(&mut state, &[key('n')]);
    hit(&mut state, KeyCode::Esc);
    assert!(!state.account_form.is_open());
}

/// Dès qu'un champ est rempli, Échap demande confirmation et Échap de nouveau revient à la saisie.
#[test]
fn escape_with_input_asks_confirmation() {
    let mut state = AppState::new(100, 30, 1);
    type_keys(&mut state, &[key('n')]);
    type_text(&mut state, "alice");
    hit(&mut state, KeyCode::Esc);
    assert!(state.account_form.form().unwrap().confirm_close);
    hit(&mut state, KeyCode::Esc);
    let form = state.account_form.form().unwrap();
    assert!(!form.confirm_close);
    assert_eq!(form.address.text(), "alice");
}

/// Entrée ou `y` sous la confirmation ferme et efface le mot de passe ; les autres touches n'écrivent rien.
#[test]
fn confirming_closes_and_a_stray_key_changes_nothing() {
    let mut state = ready_state(known_provider());
    type_text(&mut state, SECRET);
    hit(&mut state, KeyCode::Esc);
    type_text(&mut state, "xzw");
    assert_eq!(state.account_form.form().unwrap().password.char_count(), SECRET.len());
    assert_eq!(state.account_form.form().unwrap().address.text(), "alice@acme.test");
    type_text(&mut state, "y");
    assert!(!state.account_form.is_open());
    assert!(!format!("{state:?}").contains(SECRET));
}

/// Pendant que le formulaire est ouvert, les touches du courrier sont du texte et ne fuient pas vers l'arrière-plan.
#[test]
fn keys_are_text_and_do_not_leak_to_the_mail_views() {
    let mut state = AppState::new(100, 30, 1);
    type_keys(&mut state, &[key('n')]);
    let commands = type_text(&mut state, "q?/c:d");
    assert!(commands.is_empty());
    assert!(!state.should_quit);
    assert_eq!(state.account_form.form().unwrap().address.text(), "q?/c:d");
    assert!(!state.popup.is_open());
}

/// Ctrl+c quitte même avec le formulaire ouvert, comme partout.
#[test]
fn ctrl_c_still_quits() {
    let mut state = AppState::new(100, 30, 1);
    type_keys(&mut state, &[key('n')]);
    let commands = type_keys(&mut state, &[input_fixtures::ctrl('c')]);
    assert!(state.should_quit);
    assert_eq!(commands, vec![mira_tui::commands::Command::Quit]);
}
