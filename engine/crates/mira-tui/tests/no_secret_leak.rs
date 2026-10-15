//! Absence de secret dans l'écran, les sorties Debug et la ligne d'état.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod account_form_fixtures;
mod input_fixtures;

use account_form_fixtures::{
    SECRET, added_reply, hit, known_provider, login_error, ready_state, screen, token_of, type_text,
};
use crossterm::event::KeyCode;
use mira_tui::update::update;
use mira_tui::view::masked_input::mask;

/// Le mot de passe n'apparaît ni dans la sortie Debug de l'état, ni à l'écran, avant et après l'envoi.
#[test]
fn debug_output_of_state_has_no_secret() {
    let mut state = ready_state(known_provider());
    type_text(&mut state, SECRET);
    assert!(!format!("{state:?}").contains(SECRET));
    assert!(!screen(&state, 100, 30).contains(SECRET));
    let sent = hit(&mut state, KeyCode::Enter);
    assert!(!sent.is_empty());
    assert_eq!(state.account_form.form().unwrap().password.char_count(), 0);
    assert!(!format!("{state:?}").contains(SECRET));
    assert!(!screen(&state, 100, 30).contains(SECRET));
}

/// Une erreur de connexion ne conserve ni n'affiche le secret dans l'état ou l'écran.
#[test]
fn connection_error_leaves_no_secret_in_state_nor_screen() {
    let mut state = ready_state(known_provider());
    type_text(&mut state, SECRET);
    let sent = hit(&mut state, KeyCode::Enter);
    update(&mut state, added_reply(token_of(&sent), Err(login_error())));
    assert!(!format!("{state:?}").contains(SECRET));
    assert!(!screen(&state, 100, 30).contains(SECRET));
    assert!(!format!("{:?}", state.status).contains(SECRET));
}

/// Fermer le formulaire écrase et vide tout secret encore présent.
#[test]
fn closing_form_wipes_secret() {
    let mut state = ready_state(known_provider());
    type_text(&mut state, SECRET);
    state.account_form.close();
    assert!(!format!("{state:?}").contains(SECRET));
    assert!(!screen(&state, 100, 30).contains(SECRET));
}

/// Le masque d'affichage ne produit que des pastilles et jamais la valeur en clair.
#[test]
fn masked_input_shows_only_bullets() {
    let masked = mask(SECRET.len(), 50);
    assert!(!masked.contains(SECRET));
    assert!(masked.chars().all(|c| c == '•'));
    assert_eq!(masked.chars().count(), SECRET.len());
}
