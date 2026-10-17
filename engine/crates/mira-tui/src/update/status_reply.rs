//! Réponses du moteur qui alimentent la barre d'état.

use mira_assembly::api::{AccountSyncStatus, Response, SyncResponse};
use mira_assembly::mira_vocabulary::{AccountState, MiraError};

use super::accounts_reply::{report, unexpected};
use super::sync_status::refresh_status;
use crate::commands::Command;
use crate::state::AppState;
use crate::view::safe_text::display_safe;

/// Après une demande de synchro acceptée, relit l'état ; sinon montre l'erreur.
pub fn on_start_sync(state: &mut AppState, result: Result<Response, MiraError>) -> Vec<Command> {
    match result {
        Ok(_) => refresh_status(state),
        Err(error) => report(state, error),
    }
}

/// Retient le total des actions en file de tous les comptes.
pub fn on_status(state: &mut AppState, result: Result<Response, MiraError>) -> Vec<Command> {
    state.refresh.status = false;
    match result {
        Ok(Response::Status(status)) => {
            let pending = status.accounts.iter().map(|a| a.pending_actions);
            state.status.pending_actions = pending.sum();
            Vec::new()
        }
        Ok(_) => unexpected(state),
        Err(error) => report(state, error),
    }
}

/// Retient si une synchro est en cours, si un compte est hors ligne et la dernière erreur de synchro.
pub fn on_sync_status(state: &mut AppState, result: Result<Response, MiraError>) -> Vec<Command> {
    state.refresh.sync = false;
    match result {
        Ok(Response::Sync(SyncResponse::Status(accounts))) => store_sync(state, &accounts),
        Ok(_) => return unexpected(state),
        Err(error) => return report(state, error),
    }
    Vec::new()
}

/// Résume l'état de synchro de tous les comptes pour la barre d'état.
fn store_sync(state: &mut AppState, accounts: &[AccountSyncStatus]) {
    state.status.syncing = accounts.iter().any(|a| a.syncing);
    state.status.offline = accounts.iter().any(|a| a.state == AccountState::Offline);
    state.status.sync_error = accounts
        .iter()
        .find_map(|a| a.last_error.as_deref())
        .map(display_safe);
}
