#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Cycle de vie d'un compte par le CLI, contre le faux serveur IMAP du testkit.

#[allow(dead_code)]
mod cli_support;

use std::time::Duration;

use cli_support::server::{PASSWORD, USER, start_bench};
use cli_support::{account_id_of, parse_json};

/// Ajout avec mot de passe sur stdin : compte listé, secret au trousseau, aucune fuite
#[tokio::test]
async fn add_with_password_on_stdin_then_list() {
    let bench = tokio::time::timeout(Duration::from_secs(20), start_bench())
        .await
        .unwrap();
    let stdin = format!("{PASSWORD}\n");
    let added = bench
        .run(
            &stdin,
            &[
                "account",
                "add",
                "--address",
                USER,
                "--password-stdin",
                "--json",
            ],
        )
        .await;
    assert_eq!(added.code, 0, "{}", added.stderr);
    let account = account_id_of(&added);
    assert_eq!(bench.stored_password(&account).as_deref(), Some(PASSWORD));
    let listed = bench.run("", &["account", "list"]).await;
    assert_eq!(listed.code, 0);
    assert!(listed.stdout.contains(USER));
    assert!(listed.stdout.contains("connected"));
    for text in [&added.stdout, &added.stderr, &listed.stdout, &listed.stderr] {
        assert!(!text.contains(PASSWORD));
    }
}

/// Pause puis reprise : l'état change, la reprise passe par « hors ligne »
#[tokio::test]
async fn pause_then_resume_changes_state() {
    let bench = start_bench().await;
    let account = bench.add_default_account().await;
    let paused = bench
        .run("", &["account", "pause", &account, "--json"])
        .await;
    assert_eq!(paused.code, 0, "{}", paused.stderr);
    assert_eq!(
        parse_json(&paused)["data"]["data"]["state"]["type"],
        "paused"
    );
    let resumed = bench
        .run("", &["account", "resume", &account, "--json"])
        .await;
    assert_eq!(resumed.code, 0, "{}", resumed.stderr);
    assert_eq!(
        parse_json(&resumed)["data"]["data"]["state"]["type"],
        "offline"
    );
}

/// Suppression : le compte disparaît de la liste et son secret du trousseau
#[tokio::test]
async fn delete_removes_account_and_secret() {
    let bench = start_bench().await;
    let account = bench.add_default_account().await;
    let deleted = bench.run("", &["account", "delete", &account]).await;
    assert_eq!(deleted.code, 0, "{}", deleted.stderr);
    let listed = bench.run("", &["account", "list", "--json"]).await;
    assert_eq!(
        parse_json(&listed)["data"]["data"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(bench.stored_password(&account), None);
}

/// Rejouer l'ajout avec la même clé d'idempotence ne crée qu'un compte
#[tokio::test]
async fn add_replayed_with_same_key_creates_one_account() {
    let bench = start_bench().await;
    let stdin = format!("{PASSWORD}\n");
    let arguments = [
        "account",
        "add",
        "--address",
        USER,
        "--password-stdin",
        "--key",
        "k-1",
        "--json",
    ];
    let first = bench.run(&stdin, &arguments).await;
    let second = bench.run(&stdin, &arguments).await;
    assert_eq!(second.code, 0, "{}", second.stderr);
    assert_eq!(account_id_of(&first), account_id_of(&second));
    let listed = bench.run("", &["account", "list", "--json"]).await;
    assert_eq!(
        parse_json(&listed)["data"]["data"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

/// `account test` n'enregistre rien et `account prepare` propose les serveurs
#[tokio::test]
async fn test_and_prepare_register_nothing() {
    let bench = start_bench().await;
    let stdin = format!("{PASSWORD}\n");
    let tested = bench
        .run(
            &stdin,
            &["account", "test", "--address", USER, "--password-stdin"],
        )
        .await;
    assert_eq!(tested.code, 0, "{}", tested.stderr);
    let prepared = bench
        .run("", &["account", "prepare", "--address", USER])
        .await;
    assert_eq!(prepared.code, 0);
    assert!(prepared.stdout.contains("127.0.0.1"));
    assert_eq!(bench.secrets.entry_count(), 0);
}
