//! Palette de commandes `:` : saisie en cours et liste des commandes connues.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

use super::centered;
use crate::state::AppState;
use crate::theme;
use crate::update::palette::parse::COMMANDS;
use crate::view::safe_text::display_safe;

/// Noms des commandes connues, séparés par des espaces.
fn command_names() -> String {
    let names: Vec<&str> = COMMANDS.iter().map(|(name, _)| *name).collect();
    names.join(" ")
}

/// Dessine la saisie (neutralisée, avec curseur) et les commandes connues, centrées sur un fond effacé.
pub fn render_palette(frame: &mut Frame, state: &AppState) {
    let area: Rect = centered(frame.area(), 64, 6);
    frame.render_widget(Clear, area);
    let lines = vec![
        Line::from(format!(":{}▏", display_safe(state.palette.input()))),
        Line::from(""),
        Line::styled(format!("Commandes : {}", command_names()), theme::dim()),
    ];
    let block = Block::bordered()
        .title("Commande")
        .border_style(theme::border(true));
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true }).block(block);
    frame.render_widget(paragraph, area);
}
