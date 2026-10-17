//! Confirmation de suppression d'un brouillon : seul un second `D` immédiat supprime.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Paragraph, Wrap};

use crate::theme;
use crate::update::drafts::target::subject_label;

/// Nomme le brouillon (objet neutralisé) et rappelle le geste.
pub fn render_delete_draft(frame: &mut Frame, area: Rect, subject: &str) {
    let text = format!(
        "Supprimer le brouillon « {} » ? Il sera perdu.\n\
         D : supprimer · toute autre touche : annuler",
        subject_label(subject)
    );
    let block = Block::bordered()
        .title("Supprimer le brouillon")
        .border_style(theme::border(true));
    let paragraph = Paragraph::new(text).wrap(Wrap { trim: true }).block(block);
    frame.render_widget(paragraph, area);
}
