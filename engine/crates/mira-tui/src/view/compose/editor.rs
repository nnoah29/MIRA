//! Corps du brouillon : seules les lignes visibles sont construites, fenêtrées autour du curseur.

use ratatui::Frame;
use ratatui::layout::{Position, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

use super::line_window::{cursor_offset, left_edge, visible_slice};
use crate::state::compose::viewport::visible_top;
use crate::state::compose::{ComposeState, Field, TextBuffer};
use crate::theme;
use crate::view::safe_text::display_safe;

/// Dessine le corps dans un cadre ; le curseur du terminal y est posé si le corps a le focus.
pub fn render_body(frame: &mut Frame, area: Rect, compose: &ComposeState) {
    let focused = compose.focus == Field::Body;
    let title = if compose.has_unsaved() {
        "Corps — modifié"
    } else {
        "Corps"
    };
    let block = Block::bordered()
        .title(title)
        .border_style(theme::border(focused));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let buffer = compose.buffer(Field::Body);
    let (cursor_line, cursor_col) = buffer.cursor();
    let top = visible_top(buffer.top(), cursor_line, usize::from(inner.height));
    let current = buffer.lines().get(cursor_line).map_or("", String::as_str);
    let start = left_edge(current, cursor_col, usize::from(inner.width));
    frame.render_widget(Paragraph::new(window(buffer, inner, top, start)), inner);
    if focused && !inner.is_empty() {
        let offset = cursor_offset(current, start, cursor_col);
        place_cursor(frame, inner, offset, cursor_line.saturating_sub(top));
    }
}

/// Lignes visibles de la zone à partir de `top`, chacune fenêtrée à partir du caractère `start` et neutralisée.
fn window(buffer: &TextBuffer, inner: Rect, top: usize, start: usize) -> Vec<Line<'static>> {
    let width = usize::from(inner.width);
    buffer
        .lines()
        .iter()
        .skip(top)
        .take(usize::from(inner.height))
        .map(|line| Line::raw(display_safe(&visible_slice(line, start, width))))
        .collect()
}

/// Place le curseur du terminal à `column` colonnes et `row` lignes du coin de la zone, sans en sortir.
fn place_cursor(frame: &mut Frame, inner: Rect, column: usize, row: usize) {
    let x = column.min(usize::from(inner.width.saturating_sub(1)));
    let y = row.min(usize::from(inner.height.saturating_sub(1)));
    let x = inner.x + u16::try_from(x).unwrap_or(0);
    let y = inner.y + u16::try_from(y).unwrap_or(0);
    frame.set_cursor_position(Position::new(x, y));
}
