//! Écran d'accueil affiché tant qu'aucun compte n'est configuré.

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

/// Invite à ajouter un premier compte avec la touche n.
pub fn render_welcome(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from("Bienvenue dans MIRA"),
        Line::from(""),
        Line::from("Aucun compte configuré."),
        Line::from("Appuyez sur n pour ajouter un compte, ou sur q pour quitter."),
    ];
    let widget = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(Block::bordered().title("MIRA"));
    frame.render_widget(widget, area);
}
