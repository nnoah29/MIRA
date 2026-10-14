//! Rendu de l'éditeur dans un terminal de test.
#![allow(dead_code)]

use mira_tui::state::AppState;
use mira_tui::view::render;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

/// Texte affiché, ligne par ligne, et position du curseur du terminal.
pub fn screen(state: &AppState, width: u16, height: u16) -> (String, (u16, u16)) {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| render(frame, state)).unwrap();
    let cursor = terminal.get_cursor_position().unwrap();
    let buffer = terminal.backend().buffer().clone();
    let symbols: Vec<&str> = buffer.content().iter().map(|cell| cell.symbol()).collect();
    let text = symbols
        .chunks(usize::from(width.max(1)))
        .map(|row| row.concat())
        .collect::<Vec<_>>()
        .join("\n");
    (text, (cursor.x, cursor.y))
}

/// Texte seul d'un rendu 100x30.
pub fn text_of(state: &AppState) -> String {
    screen(state, 100, 30).0
}
