//! Page Comptes : adresse, nom, état et fenêtre de synchronisation de chaque compte.

use mira_assembly::mira_vocabulary::{Account, AccountState};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;

use super::rows::render_window;
use crate::state::AppState;
use crate::update::settings::account_nav::current_index;
use crate::view::safe_text::display_safe;

/// État de connexion en toutes lettres.
pub fn state_label(state: &AccountState) -> String {
    match state {
        AccountState::Connected => "connecté".to_string(),
        AccountState::Offline => "hors ligne".to_string(),
        AccountState::AuthRequired => "reconnexion requise".to_string(),
        AccountState::Paused => "en pause".to_string(),
        AccountState::Error { code } => format!("erreur ({})", display_safe(code)),
    }
}

/// Ligne d'un compte, texte du moteur neutralisé.
fn account_line(account: &Account) -> String {
    format!(
        "{} — {} — {} — {} jours",
        display_safe(account.address.as_str()),
        display_safe(&account.display_name),
        state_label(&account.state),
        account.sync.window_days
    )
}

/// Dessine la liste des comptes, ou l'invitation à en ajouter un.
pub fn render_accounts(frame: &mut Frame, area: Rect, state: &AppState) {
    let accounts = state.accounts.items();
    if accounts.is_empty() {
        let text = "Aucun compte configuré. Échap pour fermer, puis ajoutez-en un.";
        frame.render_widget(Paragraph::new(text), area);
        return;
    }
    let line = |index: usize| accounts.get(index).map(account_line).unwrap_or_default();
    render_window(frame, area, accounts.len(), current_index(state), &line);
}
