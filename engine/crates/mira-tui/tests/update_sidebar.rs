//! Chargement des comptes puis des boîtes, et gestion des erreurs.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod sidebar_fixtures;

use mira_assembly::api::{MailboxRequest, Request, Response};
use mira_assembly::mira_vocabulary::{AccountId, ErrorCategory, MailboxRole, MiraError};
use mira_tui::commands::{Command, Tag};
use mira_tui::state::AppState;
use mira_tui::update::{Msg, Reply, update};
use sidebar_fixtures::{account, mailbox};

/// Les comptes reçus sont affichés et la boîte du premier compte est demandée.
#[test]
fn accounts_reply_triggers_mailbox_load_for_first_account() {
    let mut state = AppState::new(80, 24, 1);
    let reply = Reply {
        tag: Tag::Accounts,
        result: Ok(Response::Accounts(vec![account("a1"), account("a2")])),
    };
    let commands = update(&mut state, Msg::Reply(reply));
    let first = AccountId::from_raw("a1");
    assert_eq!(state.accounts.len(), 2);
    assert_eq!(state.accounts.selection.selected, Some(0));
    assert_eq!(
        commands,
        vec![Command::Execute {
            tag: Tag::Mailboxes(first.clone()),
            request: Request::Mailbox(MailboxRequest::ListMailboxes { account: first }),
        }]
    );
}

/// Aucun compte : aucune commande de suite, pas de panique.
#[test]
fn empty_accounts_reply_triggers_nothing() {
    let mut state = AppState::new(80, 24, 1);
    let reply = Reply {
        tag: Tag::Accounts,
        result: Ok(Response::Accounts(Vec::new())),
    };
    assert!(update(&mut state, Msg::Reply(reply)).is_empty());
    assert!(state.accounts.is_empty());
}

/// Les boîtes reçues sont triées par rôle puis par nom.
#[test]
fn mailboxes_reply_sorts_by_role_then_name() {
    let mut state = AppState::new(80, 24, 1);
    state.accounts.replace_items(vec![account("a1")]);
    let boxes = vec![
        mailbox("a1", "Zeta", MailboxRole::Other, 0),
        mailbox("a1", "Sent", MailboxRole::Sent, 0),
        mailbox("a1", "Alpha", MailboxRole::Other, 0),
        mailbox("a1", "INBOX", MailboxRole::Inbox, 3),
    ];
    let reply = Reply {
        tag: Tag::Mailboxes(AccountId::from_raw("a1")),
        result: Ok(Response::Mailboxes(boxes)),
    };
    update(&mut state, Msg::Reply(reply));
    let names: Vec<&str> = state
        .mailboxes
        .items()
        .iter()
        .map(|m| m.name.as_str())
        .collect();
    assert_eq!(names, ["INBOX", "Sent", "Alpha", "Zeta"]);
    assert_eq!(state.mailboxes.selection.selected, Some(0));
}

/// Les boîtes d'un autre compte que le compte sélectionné sont ignorées.
#[test]
fn mailboxes_of_another_account_are_ignored() {
    let mut state = AppState::new(80, 24, 1);
    state.accounts.replace_items(vec![account("a1")]);
    let boxes = vec![mailbox("a2", "INBOX", MailboxRole::Inbox, 1)];
    let reply = Reply {
        tag: Tag::Mailboxes(AccountId::from_raw("a2")),
        result: Ok(Response::Mailboxes(boxes)),
    };
    update(&mut state, Msg::Reply(reply));
    assert!(state.mailboxes.is_empty());
}

/// Une erreur du moteur va à la barre d'état, sans panique ni changement de liste.
#[test]
fn error_reply_goes_to_status_line_not_panic() {
    let mut state = AppState::new(80, 24, 1);
    state.accounts.replace_items(vec![account("a1")]);
    let error = MiraError::new(
        ErrorCategory::Network,
        mira_assembly::mira_vocabulary::error::codes::network::UNREACHABLE,
        "serveur injoignable",
    );
    let reply = Reply {
        tag: Tag::Accounts,
        result: Err(error),
    };
    assert!(update(&mut state, Msg::Reply(reply)).is_empty());
    let message = state.status.message.clone().unwrap();
    assert!(message.contains("serveur injoignable"));
    assert_eq!(state.accounts.len(), 1);
}
