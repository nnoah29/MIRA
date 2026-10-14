//! Champs d'en-tête (À, Cc, Cci, Objet) : une ligne chacun, fenêtrée autour du curseur.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::line_window::{cursor_offset, left_edge, visible_slice};
use crate::state::compose::{ComposeState, Field};
use crate::theme;
use crate::view::safe_text::display_safe;

const LABEL_WIDTH: usize = 8;
const FLAG: &str = " ✗ adresse invalide";
const HEADER_FIELDS: [Field; 4] = [Field::To, Field::Cc, Field::Bcc, Field::Subject];

/// Dessine les quatre champs d'en-tête, un par ligne de la zone.
pub fn render_headers(frame: &mut Frame, area: Rect, compose: &ComposeState) {
    let rows = Layout::vertical([Constraint::Length(1); 4]).split(area);
    for (field, row) in HEADER_FIELDS.into_iter().zip(rows.iter()) {
        render_row(frame, *row, compose, field);
    }
}

/// Une ligne : nom du champ, texte fenêtré et, si besoin, le signalement d'une adresse invalide.
fn render_row(frame: &mut Frame, row: Rect, compose: &ComposeState, field: Field) {
    let flagged = compose.is_flagged(field);
    let flag_width = if flagged { FLAG.chars().count() } else { 0 };
    let width = usize::from(row.width).saturating_sub(LABEL_WIDTH + flag_width);
    let text = compose.text(field);
    let cursor = compose.cursor(field).1;
    let start = left_edge(&text, cursor, width);
    let shown = display_safe(&visible_slice(&text, start, width));
    let label_style = if compose.focus == field {
        theme::unread()
    } else {
        theme::dim()
    };
    let mut spans = vec![
        Span::styled(format!("{:<5} : ", field.label()), label_style),
        Span::raw(shown),
    ];
    if flagged {
        spans.push(Span::styled(FLAG, theme::status()));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), row);
    if compose.focus == field {
        place_cursor(frame, row, cursor_offset(&text, start, cursor));
    }
}

/// Place le curseur du terminal dans la ligne, à `offset` colonnes après le nom du champ.
fn place_cursor(frame: &mut Frame, row: Rect, offset: usize) {
    let column = offset.saturating_add(LABEL_WIDTH);
    let last = usize::from(row.width.saturating_sub(1));
    if row.width == 0 || row.height == 0 {
        return;
    }
    let x = row.x + u16::try_from(column.min(last)).unwrap_or(0);
    frame.set_cursor_position(Position::new(x, row.y));
}
