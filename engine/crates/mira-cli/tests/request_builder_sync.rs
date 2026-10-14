#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Traduction des commandes `sync` et `account add-oauth` en requêtes du contrat.

#[allow(dead_code)]
mod request_catalog;

use mira_assembly::api::{AccountRequest, CredentialInput, Request};
use request_catalog::{json_of_line, request_of_line, try_request_of_line};
use serde_json::json;

/// Sans option, `sync start` vise tous les comptes ; `--account` et `--mailbox` restreignent la portée
#[test]
fn sync_start_scope_follows_the_options() {
    let all = json_of_line(&["sync", "start"]);
    assert_eq!(all["request"]["scope"], json!({"type": "all"}));
    let account = json_of_line(&["sync", "start", "--account", "acc", "--wait"]);
    assert_eq!(
        account["request"]["scope"],
        json!({"type": "account", "account": "acc"})
    );
    let mailbox = json_of_line(&["sync", "start", "--mailbox", "box"]);
    assert_eq!(
        mailbox["request"]["scope"],
        json!({"type": "mailbox", "mailbox": "box"})
    );
}

/// Compte et boîte ensemble sont refusés à l'analyse
#[test]
fn sync_start_refuses_account_and_mailbox_together() {
    let line = ["sync", "start", "--account", "a", "--mailbox", "m"];
    assert!(try_request_of_line(&line).is_err());
}

/// Les autres commandes du domaine produisent leur requête avec leurs paramètres
#[test]
fn other_sync_commands_carry_their_parameters() {
    let pause = json_of_line(&["sync", "pause", "--account", "acc"]);
    assert_eq!(
        pause["request"],
        json!({"op": "pause_sync", "account": "acc"})
    );
    let status = json_of_line(&["sync", "status"]);
    assert_eq!(status["request"], json!({"op": "get_sync_status"}));
    let extend = json_of_line(&["sync", "extend-window", "--mailbox", "box", "--days", "45"]);
    assert_eq!(
        extend["request"],
        json!({"op": "extend_window", "mailbox": "box", "days": 45})
    );
    let resync = json_of_line(&["sync", "resync", "--mailbox", "box"]);
    assert_eq!(
        resync["request"],
        json!({"op": "resync_mailbox", "mailbox": "box"})
    );
}

/// `account add-oauth` prépare un ajout OAuth2 dont le flux sera rempli après l'autorisation
#[test]
fn add_oauth_builds_an_oauth_add_request() {
    let line = ["account", "add-oauth", "--address", "a@b.c", "--key", "k-1"];
    let Request::Account(AccountRequest::AddAccount {
        draft,
        credential,
        idempotency_key,
    }) = request_of_line(&line)
    else {
        panic!("ajout de compte attendu");
    };
    assert_eq!(draft.address, "a@b.c");
    assert_eq!(idempotency_key.0, "k-1");
    assert!(matches!(credential, CredentialInput::OAuthFlow { .. }));
}

/// `--days` est obligatoire et numérique
#[test]
fn extend_window_requires_a_numeric_day_count() {
    let missing = ["sync", "extend-window", "--mailbox", "m"];
    assert!(try_request_of_line(&missing).is_err());
    let text = ["sync", "extend-window", "--mailbox", "m", "--days", "x"];
    assert!(try_request_of_line(&text).is_err());
}
