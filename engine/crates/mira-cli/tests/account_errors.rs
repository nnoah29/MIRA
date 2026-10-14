#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Erreurs des commandes de compte : bon code de sortie, aucun secret dans les sorties.

#[allow(dead_code)]
mod cli_support;

use cli_support::parse_json;
use cli_support::server::{USER, start_bench};

/// Mot de passe refusé : code 15, rien au trousseau, secret absent des sorties
#[tokio::test]
async fn wrong_password_exits_15_without_leak() {
    let bench = start_bench().await;
    let result = bench
        .run(
            "mauvais-SECRET-pass\n",
            &["account", "add", "--address", USER, "--password-stdin"],
        )
        .await;
    assert_eq!(result.code, 15);
    assert!(result.stdout.is_empty());
    assert!(!result.stderr.is_empty());
    assert!(!result.stderr.contains("mauvais-SECRET-pass"));
    assert_eq!(bench.secrets.entry_count(), 0);
}

/// En JSON, l'erreur d'authentification est une enveloppe sur stdout
#[tokio::test]
async fn wrong_password_in_json_is_an_error_envelope() {
    let bench = start_bench().await;
    let result = bench
        .run(
            "mauvais-SECRET-pass\n",
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
    assert_eq!(result.code, 15);
    let document = parse_json(&result);
    assert_eq!(document["ok"], false);
    assert_eq!(document["error"]["category"], "authentication");
    assert!(!result.stdout.contains("mauvais-SECRET-pass"));
}

/// Sans `--password-stdin`, le mot de passe n'est jamais lu : entrée invalide (10)
#[tokio::test]
async fn missing_password_stdin_flag_is_invalid_input() {
    let bench = start_bench().await;
    let result = bench
        .run(
            "secret-qui-ne-doit-pas-etre-lu\n",
            &["account", "add", "--address", USER],
        )
        .await;
    assert_eq!(result.code, 10);
    assert!(result.stderr.contains("--password-stdin"));
}

/// Entrée standard vide : entrée invalide (10)
#[tokio::test]
async fn empty_stdin_is_invalid_input() {
    let bench = start_bench().await;
    let result = bench
        .run(
            "",
            &["account", "add", "--address", USER, "--password-stdin"],
        )
        .await;
    assert_eq!(result.code, 10);
}

/// Compte inconnu : introuvable (11) ; un seul hôte serveur : entrée invalide (10)
#[tokio::test]
async fn unknown_account_is_11_and_half_servers_is_10() {
    let bench = start_bench().await;
    let unknown = bench.run("", &["account", "delete", "inconnu"]).await;
    assert_eq!(unknown.code, 11);
    let half = bench
        .run(
            "x\n",
            &[
                "account",
                "add",
                "--address",
                USER,
                "--password-stdin",
                "--imap-host",
                "h",
            ],
        )
        .await;
    assert_eq!(half.code, 10);
}
