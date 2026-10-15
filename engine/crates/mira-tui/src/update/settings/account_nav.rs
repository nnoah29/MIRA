//! Compte visé par la vue Paramètres : retrouvé par son identifiant, jamais par un indice.

use mira_assembly::mira_vocabulary::Account;

use super::identity_actions::reload_identities;
use crate::commands::Command;
use crate::state::settings::SettingsTab;
use crate::state::{AppState, Selection};

/// Indice du compte visé dans la liste des comptes ; le premier si l'identifiant a disparu.
pub fn current_index(state: &AppState) -> Option<usize> {
    let accounts = state.accounts.items();
    if accounts.is_empty() {
        return None;
    }
    let found = state
        .settings
        .account
        .as_ref()
        .and_then(|id| accounts.iter().position(|account| &account.id == id));
    Some(found.unwrap_or(0))
}

/// Compte visé par la vue.
pub fn current_account(state: &AppState) -> Option<&Account> {
    current_index(state).and_then(|index| state.accounts.items().get(index))
}

/// Vise le compte voisin ; la liste des identités est relue si le compte change.
pub fn move_account(state: &mut AppState, step: fn(&mut Selection, usize)) -> Vec<Command> {
    let previous = current_account(state).map(|account| account.id.clone());
    let mut selection = Selection {
        selected: current_index(state),
    };
    step(&mut selection, state.accounts.len());
    let next = selection
        .selected
        .and_then(|index| state.accounts.items().get(index))
        .map(|account| account.id.clone());
    state.settings.account = next.clone();
    if next != previous && state.settings.tab == SettingsTab::Identities {
        return reload_identities(state);
    }
    Vec::new()
}
