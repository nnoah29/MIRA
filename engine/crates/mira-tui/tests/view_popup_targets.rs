//! Les fenêtres modales nomment le message qu'elles visent.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod screen_fixtures;
mod search_fixtures;
mod sidebar_fixtures;

use action_fixtures::state_with_mailboxes;
use mira_tui::update::Intent;
use navigation_fixtures::press;
use screen_fixtures::text_in;
use sidebar_fixtures::rendered_text;

/// Rend l'état dans un terminal de cette taille.
fn rendered_in(state: &mira_tui::state::AppState, width: u16, height: u16) -> String {
    text_in(state, width, height)
}

/// La confirmation nomme le message visé : objet et expéditeur, neutralisés.
#[test]
fn confirm_popup_names_targeted_message() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::DeletePermanently);
    let text = rendered_text(&state);
    assert!(text.contains("« Bonjour » de Bob"), "{text}");
}

/// Le sélecteur de déplacement nomme le message à déplacer.
#[test]
fn move_picker_names_targeted_message() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::OpenMovePicker);
    let text = rendered_in(&state, 120, 30);
    assert!(text.contains("Déplacer vers"), "{text}");
    assert!(text.contains("« Bonjour » de Bob"), "{text}");
}
