//! Confirmation d'envoi : expéditeur, destinataires et objet résumés, une ligne chacun, ajustés à la hauteur.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

use super::send_summary::{fit_summary, send_summary};
use crate::state::AppState;
use crate::theme;
use crate::view::compose::sender::sender_label;

/// Nombre de lignes du résumé complet (aucune sans brouillon ouvert).
pub fn summary_rows(state: &AppState) -> usize {
    let from = sender_label(state);
    state
        .compose
        .open()
        .map_or(0, |compose| send_summary(compose, &from).len())
}

/// Dessine la confirmation du brouillon ouvert ; sur un écran bas, l'objet et les touches restent.
pub fn render_confirm_send(frame: &mut Frame, area: Rect, state: &AppState) {
    let from = sender_label(state);
    let rows = usize::from(area.height.saturating_sub(2));
    let lines: Vec<Line> = state
        .compose
        .open()
        .map(|compose| fit_summary(compose, &from, rows))
        .unwrap_or_default()
        .into_iter()
        .map(Line::from)
        .collect();
    let block = Block::bordered()
        .title("Envoyer")
        .border_style(theme::border(true));
    frame.render_widget(Paragraph::new(lines).block(block), area);
}
