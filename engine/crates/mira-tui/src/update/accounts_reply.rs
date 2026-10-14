//! Réponse du moteur à la liste des comptes.

use mira_assembly::api::Response;
use mira_assembly::mira_vocabulary::{Account, MiraError};

use super::account_switch::{reload_if_changed, selected_id};
use crate::commands::Command;
use crate::state::AppState;
use crate::view::safe_text::display_safe;

/// Range les comptes reçus et demande les boîtes du compte sélectionné.
pub fn on_accounts(state: &mut AppState, result: Result<Response, MiraError>) -> Vec<Command> {
    match result {
        Ok(Response::Accounts(accounts)) => store_accounts(state, accounts),
        Ok(_) => unexpected(state),
        Err(error) => report(state, error),
    }
}

/// Remplace les comptes ; ne recharge les boîtes que si le compte sélectionné change.
fn store_accounts(state: &mut AppState, accounts: Vec<Account>) -> Vec<Command> {
    let previous = selected_id(state);
    state.accounts.replace_items(accounts);
    reload_if_changed(state, previous)
}

/// Montre l'erreur du moteur, neutralisée, dans la barre d'état.
pub fn report(state: &mut AppState, error: MiraError) -> Vec<Command> {
    state.status.message = Some(display_safe(&error.to_string()));
    Vec::new()
}

/// Signale une réponse qui ne correspond pas à la requête.
pub fn unexpected(state: &mut AppState) -> Vec<Command> {
    state.status.message = Some("réponse inattendue du moteur".to_string());
    Vec::new()
}
