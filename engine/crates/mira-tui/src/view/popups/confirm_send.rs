//! Confirmation d'envoi : destinataires et objet du brouillon, neutralisés et raccourcis.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Wrap};

use crate::state::compose::{ComposeState, Field};
use crate::theme;
use crate::view::queue::outbox::shown;

const RECIPIENT_FIELDS: [Field; 3] = [Field::To, Field::Cc, Field::Bcc];

/// Lignes de la confirmation : question, destinataires non vides, objet, touches.
fn send_lines(compose: &ComposeState) -> Vec<Line<'static>> {
    let mut lines = vec![Line::from("Envoyer ce message ?")];
    for field in RECIPIENT_FIELDS {
        let text = compose.text(field);
        if !text.trim().is_empty() {
            lines.push(Line::from(format!("{} : {}", field.label(), shown(&text))));
        }
    }
    let subject = shown(&compose.text(Field::Subject));
    lines.push(Line::from(format!("Objet : {subject}")));
    lines.push(Line::from("Entrée ou y : envoyer · Échap ou n : annuler"));
    lines
}

/// Dessine la confirmation du brouillon ouvert (vide s'il n'y en a pas).
pub fn render_confirm_send(frame: &mut Frame, area: Rect, compose: Option<&ComposeState>) {
    let lines = compose.map(send_lines).unwrap_or_default();
    let block = Block::bordered()
        .title("Envoyer")
        .border_style(theme::border(true));
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true }).block(block);
    frame.render_widget(paragraph, area);
}
