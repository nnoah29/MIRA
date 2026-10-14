//! Défilement par fenêtre : seules les lignes visibles sont construites, la sélection reste visible ; corps neutralisé une fois.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod screen_fixtures;
mod search_fixtures;
mod sidebar_fixtures;

use mira_tui::state::ReaderContent;
use mira_tui::update::{Intent, update};
use mira_tui::view::message_list::window::{list_rows, visible_range};
use mira_tui::view::reader::body::visible_text;
use navigation_fixtures::{press, state_with_messages};
use reader_fixtures::{body, body_reply};
use screen_fixtures::text_in;

/// La fenêtre contient la sélection et ne dépasse pas la capacité.
#[test]
fn visible_range_keeps_selection_within_capacity() {
    assert_eq!(visible_range(100_000, Some(0), 14), 0..14);
    assert_eq!(visible_range(100_000, Some(99_999), 14), 99_986..100_000);
    assert_eq!(visible_range(100_000, None, 14), 0..14);
    assert_eq!(visible_range(5, Some(4), 14), 0..5);
    assert_eq!(visible_range(0, None, 14), 0..0);
    assert_eq!(visible_range(10, Some(3), 0), 0..0);
}

/// Sur 100 000 messages chargés, le rendu ne construit qu'environ la hauteur visible de lignes.
#[test]
fn rendering_huge_list_builds_only_visible_rows() {
    let mut state = state_with_messages(100_000);
    press(&mut state, Intent::Bottom);
    let (rows, selected) = list_rows(&state, 40, 30);
    assert!(rows.len() <= 15, "{}", rows.len());
    assert_eq!(selected, Some(rows.len() - 1));
}

/// Après `G`, `g` et un défilement, le message sélectionné est à l'écran.
#[test]
fn selection_stays_visible_after_bottom_top_and_scroll() {
    let mut state = state_with_messages(100_000);
    state
        .accounts
        .replace_items(vec![sidebar_fixtures::account("a1")]);
    press(&mut state, Intent::Bottom);
    assert!(text_in(&state, 80, 30).contains("objet 99999"));
    press(&mut state, Intent::Top);
    assert!(text_in(&state, 80, 30).contains("objet 0"));
    for _ in 0..40 {
        press(&mut state, Intent::MoveDown);
    }
    assert!(text_in(&state, 80, 30).contains("objet 40"));
}

/// Le corps est neutralisé à la réception et gardé sûr dans l'état.
#[test]
fn body_is_neutralized_once_at_reception() {
    let mut state = action_fixtures::state_with_mailboxes();
    press(&mut state, Intent::Open);
    update(&mut state, body_reply("m1", Ok(body("a\u{1b}[31mb\u{7}c"))));
    assert_eq!(state.reader.content, ReaderContent::Body(body("abc")));
}

/// Seules les lignes du corps à partir du défilement, dans la hauteur, sont données au rendu.
#[test]
fn visible_text_slices_body_from_scroll() {
    let text: String = (0..1_000).map(|n| format!("Ligne {n}\n")).collect();
    let shown = visible_text(&text, 500, 10);
    assert_eq!(shown.lines().count(), 10);
    assert!(shown.starts_with("Ligne 500"));
    assert_eq!(visible_text(&text, 2_000, 10), "");
}
