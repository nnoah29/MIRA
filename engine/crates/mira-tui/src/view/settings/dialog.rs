//! Fenêtre de la vue Paramètres : formulaire ou confirmation, centrée sur un fond effacé.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

use super::form_view::{form_lines, form_title};
use crate::state::AppState;
use crate::state::settings::{Ask, Dialog};
use crate::theme;
use crate::view::popups::centered;

const DIALOG_WIDTH: u16 = 70;

/// Texte de la confirmation d'une opération de maintenance.
fn confirm_lines(ask: Ask) -> Vec<Line<'static>> {
    let what = match ask {
        Ask::PurgeCache => "Supprimer les pièces jointes téléchargées ? Le serveur les garde.",
        Ask::RebuildIndex => "Reconstruire l'index de recherche ? Cela peut être long.",
    };
    vec![
        Line::from(what),
        Line::from(""),
        Line::styled("Entrée/y : confirmer · Échap/n : annuler", theme::dim()),
    ]
}

/// Dessine la fenêtre ouverte de la vue, s'il y en a une.
pub fn render_dialog(frame: &mut Frame, area: Rect, state: &AppState) {
    let (title, lines) = match &state.settings.dialog {
        Dialog::None => return,
        Dialog::Confirm(ask) => ("Confirmation", confirm_lines(*ask)),
        Dialog::Form(form) => (form_title(&form.kind), form_lines(form)),
    };
    let height = u16::try_from(lines.len() + 4).unwrap_or(u16::MAX);
    let rect = centered(area, DIALOG_WIDTH, height);
    frame.render_widget(Clear, rect);
    let block = Block::bordered()
        .title(title)
        .border_style(theme::border(true));
    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .block(block);
    frame.render_widget(paragraph, rect);
}
