//! Une rafale d'événements de compte ne relit la liste des comptes qu'une fois à la fois.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod engine_fixtures;
mod sidebar_fixtures;

use engine_fixtures::event;
use mira_assembly::api::{AccountRequest, Request};
use mira_assembly::mira_vocabulary::{AccountId, EventKind};
use mira_tui::commands::Command;
use mira_tui::state::AppState;
use mira_tui::update::update;
use sidebar_fixtures::{loaded_state, refresh};

/// Nombre de relectures de la liste des comptes parmi ces commandes.
fn list_accounts_count(commands: &[Command]) -> usize {
    let wanted = Request::Account(AccountRequest::ListAccounts);
    commands
        .iter()
        .filter(|command| matches!(command, Command::Execute { request, .. } if *request == wanted))
        .count()
}

/// Applique un événement « compte ajouté » et compte les relectures demandées.
fn account_added(state: &mut AppState) -> usize {
    let kind = EventKind::AccountAdded {
        account: AccountId::from_raw("a9"),
    };
    list_accounts_count(&update(state, event(kind)))
}

/// Deux événements avant la réponse : une seule relecture ; après la réponse, une nouvelle est permise.
#[test]
fn burst_of_account_events_sends_one_list_accounts() {
    let mut state = loaded_state();
    assert_eq!(account_added(&mut state), 1);
    assert_eq!(account_added(&mut state), 0);
    refresh(&mut state, &["a1", "a2"]);
    assert_eq!(account_added(&mut state), 1);
}
