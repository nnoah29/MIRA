#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Erreurs des commandes `mira sync …` : codes de sortie et absence de secret.

#[allow(dead_code)]
mod cli_support;

use cli_support::full_bench::{FullBench, PASSWORD};
use cli_support::parse_json;

/// Boîte inconnue : introuvable (11), en JSON comme en texte
#[tokio::test]
async fn unknown_mailbox_is_not_found() {
    let bench = FullBench::start().await;
    bench.add_account().await;
    let text = bench
        .run_sync(&["sync", "resync", "--mailbox", "inconnue"])
        .await;
    assert_eq!(text.code, 11);
    assert!(text.stdout.is_empty());
    let json = bench
        .run_sync(&["sync", "start", "--mailbox", "inconnue", "--json"])
        .await;
    assert_eq!(json.code, 11);
    assert_eq!(parse_json(&json)["error"]["category"], "not_found");
}

/// Compte inconnu pour la pause ou le démarrage : introuvable (11)
#[tokio::test]
async fn unknown_account_is_not_found() {
    let bench = FullBench::start().await;
    let paused = bench.run(&["sync", "pause", "--account", "inconnu"]).await;
    assert_eq!(paused.code, 11);
    let started = bench
        .run_sync(&["sync", "start", "--account", "inconnu"])
        .await;
    assert_eq!(started.code, 11);
}

/// Zéro jour est une entrée invalide (10) ; compte et boîte ensemble sont refusés à l'analyse
#[tokio::test]
async fn invalid_arguments_are_invalid_input() {
    let bench = FullBench::start().await;
    let zero = bench
        .run_sync(&["sync", "extend-window", "--mailbox", "box", "--days", "0"])
        .await;
    assert_eq!(zero.code, 10);
    let both = bench
        .run(&["sync", "start", "--account", "a", "--mailbox", "m"])
        .await;
    assert_eq!(both.code, 10);
    let missing = bench
        .run(&["sync", "extend-window", "--mailbox", "m"])
        .await;
    assert_eq!(missing.code, 10);
}

/// Sans synchro de fond permise par les dépendances, la demande est refusée (12) sans rien envoyer
#[tokio::test]
async fn sync_is_refused_when_dependencies_disable_it() {
    let bench = FullBench::start().await;
    bench.add_account().await;
    let connections = bench.imap.connection_count();
    let result = bench.run(&["sync", "start"]).await;
    assert_eq!(result.code, 12, "{}", result.stderr);
    assert!(result.stderr.contains("sync_disabled"), "{}", result.stderr);
    assert_eq!(bench.imap.connection_count(), connections);
}

/// Mot de passe retiré du trousseau : la synchro échoue en authentification (15), sans fuite
#[tokio::test]
async fn missing_secret_fails_with_authentication_code() {
    let bench = FullBench::start().await;
    let account = bench.add_account().await;
    bench.forget_password(&account);
    let result = bench
        .run_sync(&["sync", "start", "--account", &account, "--timeout", "20"])
        .await;
    assert_eq!(result.code, 15, "{}", result.stderr);
    assert!(!result.stdout.contains(PASSWORD));
    assert!(!result.stderr.contains(PASSWORD));
}
