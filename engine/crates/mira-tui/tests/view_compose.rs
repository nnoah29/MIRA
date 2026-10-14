//! Rendu de l'éditeur de brouillon : champs, corps, curseur, texte hostile et petits terminaux.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use compose_fixtures::{
    compose, draft, edit, focus_on, open_reply, open_with, screen, text_of, type_in_draft,
};
use mira_assembly::mira_vocabulary::AddressEntry;
use mira_tui::state::Popup;
use mira_tui::state::compose::Field;
use mira_tui::update::{ComposeIntent, EditKey};

/// L'éditeur montre les champs, leurs valeurs, le corps et le rappel des touches.
#[test]
fn compose_screen_shows_fields_body_and_key_hints() {
    let state = open_reply();
    let text = text_of(&state);
    for expected in [
        "À",
        "Cc",
        "Cci",
        "Objet",
        "bob@example.org",
        "Re: Bonjour",
        "Corps",
        "Ctrl+s",
    ] {
        assert!(text.contains(expected), "{expected}\n{text}");
    }
    assert!(!text.contains("Messages"), "{text}");
}

/// Un corps et des champs pleins de séquences de terminal s'affichent sans aucune séquence brute.
#[test]
fn hostile_draft_text_is_shown_without_raw_sequences() {
    let body = "avant \u{1b}]0;titre\u{7}milieu \u{1b}[31mrouge\u{1b}[0m \u{9b}2J après";
    let mut hostile = draft("d1", &["bob@example.org"], "Re: \u{1b}[2Jobjet", body);
    hostile.to[0].name = Some("Eve\u{1b}]52;c;AAAA\u{7}".to_string());
    hostile.cc = vec![AddressEntry {
        name: None,
        address: "x\u{1b}[1mx@example.org".to_string(),
    }];
    let state = open_with(hostile);
    let text = text_of(&state);
    for raw in ['\u{1b}', '\u{7}', '\u{9b}', '\u{9d}'] {
        assert!(!text.contains(raw), "{raw:?}\n{text}");
    }
    assert!(text.contains("avant") && text.contains("après"), "{text}");
    assert!(text.contains("objet"), "{text}");
}

/// Des terminaux minuscules, avec ou sans fenêtre, ne font jamais paniquer le rendu.
#[test]
fn tiny_terminals_do_not_panic() {
    let long = "x".repeat(300);
    let mut states = vec![open_reply(), open_with(draft("d1", &[], &long, &long))];
    let mut asking = open_reply();
    focus_on(&mut asking, Field::Subject);
    type_in_draft(&mut asking, "!");
    compose(&mut asking, ComposeIntent::Close);
    assert_eq!(asking.popup, Popup::ConfirmCloseDraft);
    states.push(asking);
    for state in &states {
        for (width, height) in [(0, 0), (1, 1), (10, 3), (3, 2), (99, 5), (40, 8), (200, 2)] {
            screen(state, width, height);
        }
    }
}

/// Une très longue ligne est vue par une fenêtre autour du curseur, sans déborder.
#[test]
fn long_line_is_windowed_around_the_cursor() {
    let line = format!("{}FIN", "x".repeat(5_000));
    let mut state = open_with(draft("d1", &["bob@example.org"], "Re: Bonjour", &line));
    focus_on(&mut state, Field::Body);
    edit(&mut state, EditKey::End);
    let (text, cursor) = screen(&state, 40, 14);
    assert!(text.contains("xxFIN"), "{text}");
    assert!(text.lines().all(|row| row.chars().count() <= 40));
    assert!(cursor.0 < 40 && cursor.1 < 14, "{cursor:?}");
    edit(&mut state, EditKey::Home);
    let (text, _) = screen(&state, 40, 14);
    assert!(!text.contains("FIN"), "{text}");
}

/// Le curseur du terminal se place d'après la largeur d'affichage (日本語 occupe deux colonnes par caractère).
#[test]
fn cursor_follows_display_width_of_wide_chars() {
    let mut state = open_with(draft("d1", &["bob@example.org"], "Re: Bonjour", "日本語"));
    focus_on(&mut state, Field::Body);
    edit(&mut state, EditKey::End);
    let (text, cursor) = screen(&state, 60, 14);
    let row = text.lines().nth(usize::from(cursor.1)).unwrap();
    assert!(row.replace(' ', "").contains("日本語"), "{row}");
    assert_eq!(cursor.0, 1 + 6);
    edit(&mut state, EditKey::Left);
    assert_eq!(screen(&state, 60, 14).1.0, 1 + 4);
}
