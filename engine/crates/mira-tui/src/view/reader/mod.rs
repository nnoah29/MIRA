//! Colonne de droite : en-têtes, pièces jointes, fil et corps du message ouvert.

pub mod body;
pub mod fit;
pub mod header;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::{Block, Paragraph};

use crate::state::{AppState, Pane};
use crate::theme;

/// Dessine le lecteur, ou une invite si aucun message n'est ouvert.
pub fn render_reader(frame: &mut Frame, area: Rect, state: &AppState) {
    let focused = state.focus == Pane::Reader;
    let block = Block::bordered()
        .title("Lecture")
        .border_style(theme::border(focused));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let Some(message) = state.reader.message() else {
        let hint = Paragraph::new("Sélectionnez un message").style(theme::dim());
        frame.render_widget(hint, inner);
        return;
    };
    let budget = usize::from((inner.height / 2).min(inner.height.saturating_sub(3)));
    let lines = header::header_lines(message, state, usize::from(inner.width), budget);
    let height = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    let [head, text] =
        Layout::vertical([Constraint::Length(height), Constraint::Min(0)]).areas(inner);
    frame.render_widget(Paragraph::new(lines), head);
    body::render_body(frame, text, state);
}
