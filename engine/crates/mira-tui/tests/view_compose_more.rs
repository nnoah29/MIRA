//! Rendu de l'éditeur : signalement d'une adresse invalide, confirmation et gros corps.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use std::time::{Duration, Instant};

use compose_fixtures::{
    compose, draft, edit, focus_on, open_reply, open_with, screen, text_of, type_in_draft,
};
use mira_tui::state::compose::Field;
use mira_tui::update::{ComposeIntent, EditKey};

/// Une adresse invalide est marquée à l'écran, et l'erreur de la barre d'état est neutralisée.
#[test]
fn invalid_address_is_marked_on_screen() {
    let mut state = open_reply();
    focus_on(&mut state, Field::To);
    type_in_draft(&mut state, ", pas une adresse");
    compose(&mut state, ComposeIntent::Save);
    let text = text_of(&state);
    assert!(text.contains("adresse invalide"), "{text}");
    assert!(text.contains("pas une adresse"), "{text}");
}

/// La confirmation de fermeture dit les trois choix.
#[test]
fn close_confirmation_is_displayed() {
    let mut state = open_reply();
    focus_on(&mut state, Field::Subject);
    type_in_draft(&mut state, "!");
    compose(&mut state, ComposeIntent::Close);
    let text = text_of(&state);
    for expected in ["pas enregistrées", "enregistrer", "abandonner", "continuer"] {
        assert!(text.contains(expected), "{expected}\n{text}");
    }
}

/// Le focus se voit : le curseur du terminal est sur la ligne du champ qui a le focus.
#[test]
fn terminal_cursor_sits_on_the_focused_field_row() {
    let mut state = open_reply();
    focus_on(&mut state, Field::Subject);
    let (text, cursor) = screen(&state, 80, 20);
    let row = text.lines().nth(usize::from(cursor.1)).unwrap();
    assert!(row.contains("Objet"), "{row}");
    focus_on(&mut state, Field::To);
    let (text, cursor) = screen(&state, 80, 20);
    assert!(
        text.lines()
            .nth(usize::from(cursor.1))
            .unwrap()
            .contains("À")
    );
}

/// Un corps de 100 000 caractères reste fluide, et la fenêtre suit le curseur jusqu'à la ligne 1000.
#[test]
fn hundred_thousand_chars_stay_fluid_and_follow_the_cursor() {
    let body: Vec<String> = (0..2_000)
        .map(|n| format!("ligne {n:04} {}", "x".repeat(40)))
        .collect();
    let mut state = open_with(draft(
        "d1",
        &["bob@example.org"],
        "Re: Bonjour",
        &body.join("\n"),
    ));
    focus_on(&mut state, Field::Body);
    let started = Instant::now();
    for _ in 0..1_000 {
        edit(&mut state, EditKey::Down);
    }
    for _ in 0..20 {
        screen(&state, 100, 30);
    }
    let (text, _) = screen(&state, 100, 30);
    assert!(text.contains("ligne 1000 "), "{text}");
    assert!(!text.contains("ligne 0001 "), "{text}");
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "{:?}",
        started.elapsed()
    );
}
