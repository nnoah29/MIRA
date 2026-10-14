#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Politique de synchro de fond par commande : active pour la synchro, éteinte pour tout le reste.

#[allow(dead_code)]
mod cli_support;

use std::time::Duration;

use clap::Parser;
use cli_support::full_bench::FullBench;
use cli_support::sync_bench::quick_sync;
use mira_assembly::sync::BackgroundSync;
use mira_cli::arguments::Cli;
use mira_cli::commands::background_policy::background_sync_for;

/// Mode de synchro de fond que le CLI retient pour cette ligne de commande.
fn mode_of(line: &[&str]) -> BackgroundSync {
    let mut args = vec!["mira"];
    args.extend_from_slice(line);
    let cli = Cli::try_parse_from(args).unwrap();
    background_sync_for(&cli.command, quick_sync())
}

/// Seules les commandes de synchro (et l'ajout qui attend la synchro) activent la synchro de fond
#[test]
fn only_sync_commands_enable_background_sync() {
    let on: &[&[&str]] = &[
        &["sync", "start"],
        &["sync", "extend-window", "--mailbox", "m", "--days", "3"],
        &["sync", "resync", "--mailbox", "m"],
        &["account", "add", "--address", "a@b.c", "--sync-wait"],
        &["account", "add-oauth", "--address", "a@b.c", "--sync-wait"],
    ];
    for line in on {
        assert!(matches!(mode_of(line), BackgroundSync::On(_)), "{line:?}");
    }
}

/// Toutes les autres commandes éteignent la synchro de fond, même si les dépendances la permettent
#[test]
fn every_other_command_disables_background_sync() {
    let off: &[&[&str]] = &[
        &["sync", "status"],
        &["sync", "pause", "--account", "a"],
        &["account", "list"],
        &["account", "add", "--address", "a@b.c"],
        &["account", "add-oauth", "--address", "a@b.c"],
        &["account", "pause", "a"],
        &["mailbox", "list", "a"],
        &["message", "list", "--mailbox", "m"],
        &["action", "undo", "x"],
        &["queue", "list", "a"],
        &["engine", "status"],
        &["events"],
    ];
    for line in off {
        assert!(matches!(mode_of(line), BackgroundSync::Off), "{line:?}");
    }
}

/// `account list` et les lectures locales n'ouvrent aucune connexion IMAP, même avec la synchro permise
#[tokio::test]
async fn read_commands_start_no_background_task() {
    let bench = FullBench::start().await;
    let account = bench.add_account().await;
    let connections = bench.imap.connection_count();
    let lines: &[&[&str]] = &[
        &["account", "list"],
        &["mailbox", "list", &account],
        &["sync", "status"],
        &["queue", "list", &account],
        &["sync", "pause", "--account", &account],
    ];
    for line in lines {
        let result = bench.run_sync(line).await;
        assert_eq!(result.code, 0, "{line:?} : {}", result.stderr);
    }
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(bench.imap.connection_count(), connections);
}

/// Contrôle positif : `sync start` ouvre bien des connexions au faux serveur
#[tokio::test]
async fn sync_start_does_open_connections() {
    let bench = FullBench::start().await;
    bench.add_account().await;
    let connections = bench.imap.connection_count();
    bench.run_sync_ok(&["sync", "start"]).await;
    assert!(bench.imap.connection_count() > connections);
}

/// `account add` sans `--sync-wait` ne synchronise pas : le miroir reste vide ; avec l'option, il est garni
#[tokio::test]
async fn account_add_syncs_only_with_sync_wait() {
    let bench = FullBench::start().await;
    bench.deliver_subject("INBOX", "Premier");
    let mut deps = bench.sync_deps("pa55-SECRET-xyz\n");
    let plain = ["account", "add", "--address", cli_support::full_bench::USER];
    let mut line = plain.to_vec();
    line.extend(["--password-stdin", "--key", "k-sync", "--json"]);
    let added = cli_support::run_with(&mut deps, &line).await;
    assert_eq!(added.code, 0, "{}", added.stderr);
    let account = cli_support::account_id_of(&added);
    let before = bench.run_ok(&["mailbox", "list", &account]).await;
    assert!(before["data"].as_array().unwrap().is_empty());
    let mut waiting = bench.sync_deps("pa55-SECRET-xyz\n");
    line.extend(["--sync-wait", "--timeout", "30"]);
    let again = cli_support::run_with(&mut waiting, &line).await;
    assert_eq!(again.code, 0, "{}", again.stderr);
    let inbox = bench.mailbox_named(&account, "INBOX").await;
    assert_eq!(bench.subjects_of(&inbox).await, ["Premier"]);
}
