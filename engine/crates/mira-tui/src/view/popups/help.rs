//! Fenêtre d'aide : touches du panneau sous la fenêtre, puis touches valables partout.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use crate::input::hints::{GLOBAL, Hint, context_name, hints_for};
use crate::input::pane_context;
use crate::state::AppState;
use crate::theme;

/// Ligne d'aide : touches alignées, puis leur effet.
fn hint_line(hint: &Hint) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {:<14}", hint.keys), theme::unread()),
        Span::raw(hint.action),
    ])
}

/// Lignes de l'aide pour l'état courant : contexte du panneau, puis touches globales.
pub fn help_lines(state: &AppState) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = hints_for(pane_context(state))
        .iter()
        .map(hint_line)
        .collect();
    lines.push(Line::from(""));
    lines.push(Line::styled("Partout", theme::dim()));
    lines.extend(GLOBAL.iter().map(hint_line));
    lines
}

/// Dessine l'aide dans la zone donnée, titrée du nom du contexte.
pub fn render_help(frame: &mut Frame, area: Rect, state: &AppState) {
    let title = format!("Aide — {}", context_name(pane_context(state)));
    let block = Block::bordered()
        .title(title)
        .border_style(theme::border(true));
    frame.render_widget(Paragraph::new(help_lines(state)).block(block), area);
}
