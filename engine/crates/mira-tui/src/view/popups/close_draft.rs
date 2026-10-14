//! Confirmation de fermeture d'un brouillon qui a des modifications non enregistrées.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Paragraph, Wrap};

use crate::theme;

/// Dit ce qui est en jeu et les trois choix.
pub fn render_close_draft(frame: &mut Frame, area: Rect) {
    let text = "Des modifications ne sont pas enregistrées.\n\
                s ou Entrée : enregistrer et fermer · d : abandonner · c ou Échap : continuer";
    let block = Block::bordered()
        .title("Fermer le brouillon")
        .border_style(theme::border(true));
    let paragraph = Paragraph::new(text).wrap(Wrap { trim: true }).block(block);
    frame.render_widget(paragraph, area);
}
