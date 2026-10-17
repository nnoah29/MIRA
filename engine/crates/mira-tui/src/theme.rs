//! Styles de l'interface, centralisés pour changer l'apparence en un seul endroit.

use ratatui::style::{Color, Modifier, Style};

/// Style d'un élément sélectionné.
pub fn selected() -> Style {
    Style::default().add_modifier(Modifier::REVERSED)
}

/// Style du nombre de messages non lus.
pub fn unread() -> Style {
    Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD)
}

/// Style d'un texte secondaire.
pub fn dim() -> Style {
    Style::default().fg(Color::DarkGray)
}

/// Style de la bordure d'un panneau selon qu'il a le focus.
pub fn border(focused: bool) -> Style {
    if focused {
        Style::default().fg(Color::Cyan)
    } else {
        dim()
    }
}

/// Style du message d'erreur ou d'information de la barre d'état.
pub fn status() -> Style {
    Style::default().fg(Color::Red)
}
