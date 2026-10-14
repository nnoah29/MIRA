//! Cadre provisoire de la colonne de lecture, avant sa vraie vue.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Block;

use crate::state::{AppState, Pane};
use crate::theme;

/// Cadre vide du lecteur.
pub fn render_reader_placeholder(frame: &mut Frame, area: Rect, state: &AppState) {
    let focused = state.focus == Pane::Reader;
    let block = Block::bordered()
        .title("Lecture")
        .border_style(theme::border(focused));
    frame.render_widget(block, area);
}
