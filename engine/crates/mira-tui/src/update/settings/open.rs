//! Ouverture, fermeture et changement d'onglet de la vue Paramètres.

use super::events_log::load_first_page;
use super::identity_actions::reload_identities;
use super::maintenance::load_stats;
use crate::commands::Command;
use crate::state::AppState;
use crate::state::settings::{Dialog, SettingsTab};
use crate::update::account_switch::selected_id;

/// Ouvre la vue sur cet onglet, visant le compte sélectionné dans la barre latérale.
pub fn open_settings(state: &mut AppState, tab: SettingsTab) -> Vec<Command> {
    let account = selected_id(state);
    state.settings.open(tab, account);
    load_tab(state, tab)
}

/// Referme la vue et ses fenêtres ; les lectures encore attendues seront ignorées.
pub fn close_settings(state: &mut AppState) {
    state.settings.close();
}

/// Passe à cet onglet et lit ce qu'il affiche.
pub fn switch_tab(state: &mut AppState, tab: SettingsTab) -> Vec<Command> {
    state.settings.set_tab(tab);
    load_tab(state, tab)
}

/// Lectures que demande l'affichage d'un onglet.
pub fn load_tab(state: &mut AppState, tab: SettingsTab) -> Vec<Command> {
    match tab {
        SettingsTab::Accounts => Vec::new(),
        SettingsTab::Identities => reload_identities(state),
        SettingsTab::Maintenance => load_stats(state),
        SettingsTab::Events => load_first_page(state),
    }
}

/// Échap : ferme la fenêtre ouverte, sinon la vue.
pub fn back(state: &mut AppState) -> Vec<Command> {
    if state.settings.dialog == Dialog::None {
        close_settings(state);
    } else {
        state.settings.dialog = Dialog::None;
    }
    Vec::new()
}
