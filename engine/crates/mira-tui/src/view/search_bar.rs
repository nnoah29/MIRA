//! Ligne de saisie de la recherche, affichée à la place de la barre d'état.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;

use crate::state::AppState;
use crate::theme;
use crate::view::safe_text::display_safe;

/// Montre la requête en cours de frappe, neutralisée, avec un curseur.
pub fn render_search_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    let text = format!("/ {}▏", display_safe(state.search.input()));
    frame.render_widget(Paragraph::new(Span::styled(text, theme::selected())), area);
}
