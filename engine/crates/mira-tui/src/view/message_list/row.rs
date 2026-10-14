//! Ligne d'un message : pastille non lu, expéditeur et date, puis objet.

use mira_assembly::mira_vocabulary::MessageSummary;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::ListItem;

use super::short_date::short_date;
use crate::theme;
use crate::view::safe_text::display_safe;

const DATE_WIDTH: usize = 11;

/// Coupe le texte à `max` caractères en ajoutant une ellipse si besoin.
fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let kept: String = text.chars().take(max.saturating_sub(1)).collect();
    format!("{kept}…")
}

/// Nom ou adresse du premier expéditeur, neutralisé.
pub fn sender_text(message: &MessageSummary) -> String {
    match message.from.first() {
        Some(entry) => match entry.name.as_deref().filter(|name| !name.trim().is_empty()) {
            Some(name) => display_safe(name),
            None => display_safe(&entry.address),
        },
        None => String::new(),
    }
}

/// Première ligne : pastille, expéditeur, puis date alignée à droite.
fn head_line(message: &MessageSummary, width: usize) -> Line<'static> {
    let dot = if message.flags.seen { "  " } else { "● " };
    let date = message.date.or(message.internal_date).map(short_date);
    let room = width.saturating_sub(2 + DATE_WIDTH + 1);
    let sender = truncate(&sender_text(message), room);
    let gap = " ".repeat(room.saturating_sub(sender.chars().count()) + 1);
    Line::from(vec![
        Span::styled(dot, theme::unread()),
        Span::raw(sender),
        Span::raw(gap),
        Span::styled(date.unwrap_or_default(), theme::dim()),
    ])
}

/// Élément de liste à deux lignes pour un message, pour une largeur utile donnée.
pub fn message_item(message: &MessageSummary, width: usize) -> ListItem<'static> {
    let subject = truncate(&display_safe(&message.subject), width.saturating_sub(2));
    let lines = vec![
        head_line(message, width),
        Line::from(format!("  {subject}")),
    ];
    ListItem::new(Text::from(lines))
}
