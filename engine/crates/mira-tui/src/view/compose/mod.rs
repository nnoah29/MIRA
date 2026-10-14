//! Écran de rédaction : champs d'en-tête puis corps, à la place des trois panneaux.

pub mod editor;
pub mod header_fields;
pub mod line_window;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};

use crate::state::compose::ComposeState;
use crate::state::compose::viewport::HEADER_ROWS;

/// Dessine les champs d'en-tête sur les premières lignes et le corps dans le reste de la zone.
pub fn render_compose(frame: &mut Frame, area: Rect, compose: &ComposeState) {
    let [head, body] =
        Layout::vertical([Constraint::Length(HEADER_ROWS), Constraint::Min(0)]).areas(area);
    header_fields::render_headers(frame, head, compose);
    editor::render_body(frame, body, compose);
}
