//! Décision sur un envoi incertain : le message a peut-être déjà été reçu, l'utilisateur choisit.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Wrap};

use crate::theme;
use crate::view::queue::outbox::shown;

/// Dit l'enjeu (doublon ou perte), l'objet du message et les trois touches.
pub fn render_uncertain(frame: &mut Frame, area: Rect, subject: &str) {
    let lines = vec![
        Line::from("Le message a peut-être été envoyé."),
        Line::from(format!("« {} »", shown(subject))),
        Line::from("Le renvoyer peut créer un doublon chez le destinataire."),
        Line::from("r : renvoyer · n : ne pas renvoyer · Échap : plus tard"),
    ];
    let block = Block::bordered()
        .title("Envoi incertain")
        .border_style(theme::border(true));
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true }).block(block);
    frame.render_widget(paragraph, area);
}
