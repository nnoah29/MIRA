//! Mise en page selon la taille du terminal.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod sidebar_fixtures;

use mira_tui::state::AppState;
use mira_tui::view::render;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use sidebar_fixtures::account;

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
