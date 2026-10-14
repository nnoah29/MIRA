//! Barre d'état : dernier message ou rappel des touches.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;

use crate::state::AppState;
use crate::theme;
use crate::view::safe_text::display_safe;

const HINT: &str = "Tab panneau · j/k déplacer · Entrée ouvrir · Échap retour · q quitter";

/// Affiche le message de statut, ou l'aide courte en son absence.
pub fn render_status(frame: &mut Frame, area: Rect, state: &AppState) {
    let span = match &state.status.message {
        Some(message) => Span::styled(display_safe(message), theme::status()),
        None => Span::styled(HINT, theme::dim()),
    };
    frame.render_widget(Paragraph::new(span), area);
}
