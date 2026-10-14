//! Confirmation de la suppression définitive.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Paragraph, Wrap};

use crate::theme;

/// Demande de confirmation pour ce nombre de messages.
pub fn render_confirm(frame: &mut Frame, area: Rect, count: usize) {
    let text = format!(
        "Supprimer définitivement {count} message(s) ?\nEntrée : confirmer · Échap : annuler"
    );
    let block = Block::bordered()
        .title("Confirmation")
        .border_style(theme::border(true));
    let paragraph = Paragraph::new(text).wrap(Wrap { trim: true }).block(block);
    frame.render_widget(paragraph, area);
}
