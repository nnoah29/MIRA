//! Mise en page selon la taille du terminal.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod screen_fixtures;
mod search_fixtures;
mod sidebar_fixtures;

use mira_tui::state::AppState;
use mira_tui::update::Intent;
use mira_tui::view::layout::{LayoutMode, mode_for};
use mira_tui::view::render;
use navigation_fixtures::press;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use screen_fixtures::{all_screens, text_in};
use sidebar_fixtures::{account, loaded_state};

/// Un terminal minuscule ne fait pas paniquer le rendu.
#[test]
fn tiny_terminal_does_not_panic() {
    let mut state = AppState::new(100, 30, 1);
    state.accounts.replace_items(vec![account("a1")]);
    let mut terminal = Terminal::new(TestBackend::new(3, 2)).unwrap();
    terminal.draw(|frame| render(frame, &state)).unwrap();
}

/// Sous 100 colonnes, seul le panneau actif est dessiné.
#[test]
fn narrow_terminal_shows_only_focused_pane() {
    let mut state = AppState::new(80, 24, 1);
    state.accounts.replace_items(vec![account("a1")]);
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| render(frame, &state)).unwrap();
    let cells: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect();
    assert!(cells.contains("a1@example.org"));
    assert!(!cells.contains("Messages"));
}

/// Le mode étroit commence sous 100 colonnes.
#[test]
fn layout_mode_switches_at_one_hundred_columns() {
    assert_eq!(mode_for(0), LayoutMode::Narrow);
    assert_eq!(mode_for(99), LayoutMode::Narrow);
    assert_eq!(mode_for(100), LayoutMode::Wide);
    assert_eq!(mode_for(250), LayoutMode::Wide);
}

/// En mode étroit, Entrée descend d'un panneau à la fois et Échap remonte, un seul panneau visible.
#[test]
fn narrow_terminal_shows_single_pane() {
    let mut state = loaded_state();
    let sidebar = text_in(&state, 80, 24);
    assert!(
        sidebar.contains("Comptes") && !sidebar.contains("Messages"),
        "{sidebar}"
    );
    press(&mut state, Intent::Open);
    let list = text_in(&state, 80, 24);
    assert!(
        list.contains("Messages") && !list.contains("Comptes"),
        "{list}"
    );
    assert!(!list.contains("Lecture"), "{list}");
    press(&mut state, Intent::Back);
    let back = text_in(&state, 80, 24);
    assert!(
        back.contains("Comptes") && !back.contains("Messages"),
        "{back}"
    );
    let wide = text_in(&state, 120, 30);
    assert!(wide.contains("Comptes") && wide.contains("Messages") && wide.contains("Lecture"));
}

/// Aucun écran ni aucune fenêtre ne fait paniquer le rendu sur 0×0, 1×1 ou 10×3.
#[test]
fn render_does_not_panic_on_tiny_terminals() {
    for (name, state) in all_screens() {
        for (width, height) in [(0, 0), (1, 1), (10, 3), (3, 2), (99, 5)] {
            let outcome = std::panic::catch_unwind(|| text_in(&state, width, height));
            assert!(outcome.is_ok(), "{name} en {width}x{height}");
        }
    }
}

/// Le message ouvert reste lisible en mode étroit : le lecteur occupe tout l'écran.
#[test]
fn narrow_reader_takes_whole_screen() {
    let mut state = action_fixtures::state_with_mailboxes();
    press(&mut state, Intent::Open);
    let text = text_in(&state, 60, 20);
    assert!(
        text.contains("Lecture") && text.contains("Bonjour"),
        "{text}"
    );
    assert!(!text.contains("Messages"), "{text}");
}
