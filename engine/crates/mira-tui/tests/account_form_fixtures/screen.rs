//! Rendu de l'état dans un terminal de test, pour lire ce que l'utilisateur voit.
#![allow(dead_code)]

use mira_tui::state::AppState;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

/// Texte affiché, ligne par ligne, dans un terminal de cette taille.
pub fn screen(state: &AppState, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|frame| mira_tui::view::render(frame, state))
        .unwrap();
    let buffer = terminal.backend().buffer().clone();
    let symbols: Vec<&str> = buffer.content().iter().map(|cell| cell.symbol()).collect();
    symbols
        .chunks(usize::from(width.max(1)))
        .map(|row| row.concat())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Position du curseur du terminal après le rendu, s'il est affiché.
pub fn cursor_after_render(state: &AppState, width: u16, height: u16) -> Option<(u16, u16)> {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|frame| mira_tui::view::render(frame, state))
        .unwrap();
    let position = terminal.get_cursor_position().ok()?;
    Some((position.x, position.y))
}
