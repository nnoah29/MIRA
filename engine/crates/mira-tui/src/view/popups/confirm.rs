//! Confirmation de la suppression définitive.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Paragraph, Wrap};

use mira_assembly::mira_vocabulary::MessageSummary;

use super::target_label::target_label;
use crate::theme;

/// Demande de confirmation qui nomme les messages visés.
pub fn render_confirm(frame: &mut Frame, area: Rect, messages: &[MessageSummary]) {
    let text = format!(
        "Supprimer définitivement {} message(s) ?\n{}\nEntrée : confirmer · Échap : annuler",
        messages.len(),
        target_label(messages)
    );
    let block = Block::bordered()
        .title("Confirmation")
        .border_style(theme::border(true));
    let paragraph = Paragraph::new(text).wrap(Wrap { trim: true }).block(block);
    frame.render_widget(paragraph, area);
}
