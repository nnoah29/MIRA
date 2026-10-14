//! Démarrage et réponses inattendues du moteur.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use mira_assembly::api::{AccountRequest, Request, Response};
use mira_tui::commands::{Command, Tag};
use mira_tui::state::AppState;
use mira_tui::update::{Msg, Reply, update};

/// Au démarrage, l'état demande la liste des comptes.
#[test]
fn boot_commands_load_accounts() {
    let commands = AppState::new(80, 24).boot_commands();
    assert_eq!(
        commands,
        vec![Command::Execute {
            tag: Tag::Accounts,
            request: Request::Account(AccountRequest::ListAccounts),
        }]
    );
}

/// Une réponse de forme inattendue est signalée dans la barre d'état.
#[test]
fn unexpected_response_goes_to_status_line() {
    let mut state = AppState::new(80, 24);
    let reply = Reply {
        tag: Tag::Accounts,
        result: Ok(Response::Mailboxes(Vec::new())),
    };
    assert!(update(&mut state, Msg::Reply(reply)).is_empty());
    assert!(state.status.message.is_some());
}
