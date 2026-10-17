//! Fermeture d'un brouillon : état exact de l'enregistrement et choix ; abandon confirmé dans une seconde fenêtre.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Paragraph, Wrap};

use crate::state::compose::ComposeState;
use crate::theme;

/// Ce que l'utilisateur risque : modifications non enregistrées, enregistrement en vol, ou rien.
fn stake(compose: Option<&ComposeState>) -> &'static str {
    match compose {
        Some(compose) if compose.has_dirty_fields() => {
            "Des modifications ne sont pas enregistrées."
        }
        Some(compose) if compose.is_saving() => {
            "Enregistrement en cours ; rien d'autre n'est modifié."
        }
        Some(_) | None => "Tout est enregistré.",
    }
}

/// Dit ce qui est en jeu et les trois choix.
pub fn render_close_draft(frame: &mut Frame, area: Rect, compose: Option<&ComposeState>) {
    let text = format!(
        "{}\ns ou Entrée : enregistrer et fermer · D : abandonner · c ou Échap : continuer",
        stake(compose)
    );
    let block = Block::bordered()
        .title("Fermer le brouillon")
        .border_style(theme::border(true));
    let paragraph = Paragraph::new(text).wrap(Wrap { trim: true }).block(block);
    frame.render_widget(paragraph, area);
}

/// Seconde confirmation : l'abandon perd le texte non enregistré ; un brouillon déjà enregistré garde sa dernière version.
pub fn render_discard_draft(frame: &mut Frame, area: Rect, compose: Option<&ComposeState>) {
    let stake = if compose.is_some_and(|c| c.saved_once()) {
        "Abandonner les modifications ? La version enregistrée est gardée."
    } else {
        "Abandonner le brouillon ? Le texte non enregistré sera perdu."
    };
    let text = format!("{stake}\nD : abandonner · toute autre touche : revenir à l'éditeur");
    let block = Block::bordered()
        .title("Abandonner")
        .border_style(theme::border(true));
    let paragraph = Paragraph::new(text).wrap(Wrap { trim: true }).block(block);
    frame.render_widget(paragraph, area);
}
