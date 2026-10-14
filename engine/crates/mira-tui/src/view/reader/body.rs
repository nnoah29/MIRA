//! Corps du message ouvert : texte déjà neutralisé à la réception, seule la partie visible est repliée.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Paragraph, Wrap};

use crate::state::{AppState, ReaderContent};
use crate::theme;
use crate::view::safe_text::display_safe_multiline;

const OFFLINE_HINT: &str = "Corps indisponible hors ligne — Entrée pour le télécharger";

/// Texte à montrer pour le contenu courant.
fn content_text(content: &ReaderContent) -> String {
    match content {
        ReaderContent::Empty => String::new(),
        ReaderContent::Loading => "Chargement…".to_string(),
        ReaderContent::Body(body) => body.text.clone(),
        ReaderContent::OfflineMissing => OFFLINE_HINT.to_string(),
        ReaderContent::Failed(message) => {
            format!(
                "{} — Entrée pour réessayer",
                display_safe_multiline(message)
            )
        }
    }
}

/// Dessine le corps avec le défilement de l'état.
pub fn render_body(frame: &mut Frame, area: Rect, state: &AppState) {
    let style = match state.reader.content {
        ReaderContent::Body(_) => ratatui::style::Style::default(),
        ReaderContent::Empty
        | ReaderContent::Loading
        | ReaderContent::OfflineMissing
        | ReaderContent::Failed(_) => theme::dim(),
    };
    let text = content_text(&state.reader.content);
    let shown = visible_text(
        &text,
        usize::from(state.reader.scroll),
        usize::from(area.height),
    );
    let paragraph = Paragraph::new(shown)
        .style(style)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

/// Lignes du texte à partir de la ligne `scroll`, au plus `height` (chacune occupe au moins une rangée).
pub fn visible_text(text: &str, scroll: usize, height: usize) -> String {
    text.lines()
        .skip(scroll)
        .take(height)
        .collect::<Vec<_>>()
        .join("\n")
}
