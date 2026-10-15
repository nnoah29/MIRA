//! Expéditeur visible (« De : ») dans l'éditeur et la confirmation ; résumé lisible sur un écran bas.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use compose_fixtures::{compose, open_with, screen};
use mira_assembly::mira_vocabulary::{AccountId, AddressEntry};
use mira_tui::state::AppState;
use mira_tui::update::ComposeIntent;
use sidebar_fixtures::account;

/// Éditeur d'un brouillon du compte a2 alors que deux comptes existent (a1 sélectionné).
fn draft_of_second_account() -> AppState {
    let mut draft = compose_fixtures::draft("d1", &["bob@example.org"], "Re: Bonjour", "Corps");
    draft.account = AccountId::from_raw("a2");
    let mut state = open_with(draft);
    state
        .accounts
        .replace_items(vec![account("a1"), account("a2")]);
    state
}

/// L'en-tête de l'éditeur et la confirmation nomment le compte qui envoie.
#[test]
fn sender_is_shown_in_editor_and_confirmation() {
    let mut state = draft_of_second_account();
    let text = screen(&state, 100, 30).0;
    let from = text.lines().find(|l| l.contains("De")).unwrap_or_default();
    assert!(from.contains("a2@example.org"), "{text}");
    compose(&mut state, ComposeIntent::Send);
    let text = screen(&state, 100, 30).0;
    assert!(text.contains("De : a2@example.org"), "{text}");
}

/// Confirmation chargée sur un écran bas : l'objet et les touches restent visibles (80×8, 80×5).
#[test]
fn low_screen_keeps_subject_and_keys() {
    let mut draft = compose_fixtures::draft("d1", &[], "Re: Bonjour", "Corps");
    let entry = |i: usize| AddressEntry {
        name: None,
        address: format!("x{i}@example.org"),
    };
    draft.to = (0..8).map(entry).collect();
    draft.cc = (8..12).map(entry).collect();
    draft.bcc = (12..20).map(entry).collect();
    let mut state = open_with(draft);
    compose(&mut state, ComposeIntent::Send);
    for (width, height) in [(80, 8), (80, 5)] {
        let text = screen(&state, width, height).0;
        assert!(
            text.contains("Objet : Re: Bonjour"),
            "{width}×{height}\n{text}"
        );
        assert!(
            text.contains("Entrée ou y : envoyer"),
            "{width}×{height}\n{text}"
        );
    }
}
