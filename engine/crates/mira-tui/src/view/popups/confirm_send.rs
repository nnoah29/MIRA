//! Confirmation d'envoi : résumé des destinataires et de l'objet, une ligne chacun, sans retour à la ligne.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

use super::send_summary::send_summary;
use crate::state::compose::ComposeState;
use crate::theme;

/// Nombre de lignes du résumé pour ce brouillon (aucune sans brouillon ouvert).
pub fn summary_rows(compose: Option<&ComposeState>) -> usize {
    compose.map_or(0, |compose| send_summary(compose).len())
}

/// Dessine la confirmation du brouillon ouvert (vide s'il n'y en a pas).
pub fn render_confirm_send(frame: &mut Frame, area: Rect, compose: Option<&ComposeState>) {
    let lines: Vec<Line> = compose
        .map(send_summary)
        .unwrap_or_default()
        .into_iter()
        .map(Line::from)
        .collect();
    let block = Block::bordered()
        .title("Envoyer")
        .border_style(theme::border(true));
    frame.render_widget(Paragraph::new(lines).block(block), area);
}
