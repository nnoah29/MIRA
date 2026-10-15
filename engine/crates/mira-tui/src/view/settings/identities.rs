//! Page Identités : adresse, nom et signature des identités du compte visé, l'identité par défaut marquée.

use mira_assembly::mira_vocabulary::Identity;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::Paragraph;

use super::rows::render_window;
use crate::state::AppState;
use crate::update::settings::account_nav::current_account;
use crate::view::safe_text::display_safe;

const SIGNATURE_CHARS: usize = 40;

/// Ligne d'une identité : étoile si elle est par défaut, adresse, nom, début de signature.
fn identity_line(identity: &Identity) -> String {
    let star = if identity.is_default { "★" } else { " " };
    let mut line = format!(
        "{star} {} — {}",
        display_safe(identity.address.as_str()),
        display_safe(&identity.display_name)
    );
    if let Some(signature) = &identity.signature {
        let cut: String = signature.chars().take(SIGNATURE_CHARS).collect();
        line.push_str(&format!(" — {}", display_safe(&cut)));
    }
    line
}

/// Dessine les identités du compte visé, ou l'état vide qui convient.
pub fn render_identities(frame: &mut Frame, area: Rect, state: &AppState) {
    let Some(account) = current_account(state) else {
        frame.render_widget(Paragraph::new("Aucun compte"), area);
        return;
    };
    let [head, body] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);
    let title = format!("Identités de {}", display_safe(account.address.as_str()));
    frame.render_widget(Paragraph::new(title), head);
    let identities = &state.settings.identities;
    if identities.items().is_empty() {
        let text = if identities.is_loaded() {
            "Aucune identité pour ce compte (a pour en ajouter)"
        } else {
            "Chargement…"
        };
        frame.render_widget(Paragraph::new(text), body);
        return;
    }
    let items = identities.items();
    let line = |index: usize| items.get(index).map(identity_line).unwrap_or_default();
    render_window(frame, body, items.len(), identities.selection.selected, &line);
}
