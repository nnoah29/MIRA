#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `--json` : enveloppe versionnée qui reflète exactement la réponse du contrat.

#[allow(dead_code)]
mod cli_support;

use cli_support::parse_json;
use cli_support::server::{PASSWORD, USER, start_bench};

/// `engine info --json` : version du contrat dans l'enveloppe et dans la réponse
#[tokio::test]
async fn engine_info_json_is_a_versioned_envelope() {
    let bench = start_bench().await;
    let result = bench.run("", &["engine", "info", "--json"]).await;
    assert_eq!(result.code, 0, "{}", result.stderr);
    let document = parse_json(&result);
    assert_eq!(document["contract_version"]["major"], 1);
    assert_eq!(document["ok"], true);
    assert_eq!(document["data"]["type"], "contract_info");
    assert_eq!(document["data"]["data"]["version"]["major"], 1);
}

/// `engine status` : le profil choisi par `--profile` apparaît dans l'état
#[tokio::test]
async fn profile_option_selects_the_profile() {
    let bench = start_bench().await;
    let result = bench
        .run("", &["--profile", "travail", "engine", "status", "--json"])
        .await;
    assert_eq!(result.code, 0, "{}", result.stderr);
    assert_eq!(parse_json(&result)["data"]["data"]["profile"], "travail");
}

/// `events --after` rend les événements suivants, numérotés, sans secret
#[tokio::test]
async fn events_after_returns_later_events_without_secret() {
    let bench = start_bench().await;
    let stdin = format!("{PASSWORD}\n");
    bench
        .run(
            &stdin,
            &["account", "add", "--address", USER, "--password-stdin"],
        )
        .await;
    let all = bench.run("", &["events", "--json"]).await;
    let events = parse_json(&all)["data"]["data"].as_array().unwrap().clone();
    assert!(events.len() >= 2);
    let first = events[0]["seq"].as_u64().unwrap();
    let later = bench
        .run("", &["events", "--after", &first.to_string(), "--json"])
        .await;
    let rest = parse_json(&later)["data"]["data"]
        .as_array()
        .unwrap()
        .clone();
    assert!(!rest.is_empty());
    assert!(
        rest.iter()
            .all(|event| event["seq"].as_u64().unwrap() > first)
    );
    assert!(!all.stdout.contains(PASSWORD));
    let human = bench.run("", &["events"]).await;
    assert!(human.stdout.contains("SEQ"));
}

/// Identités : liste, ajout, passage par défaut, rendu humain
#[tokio::test]
async fn identity_list_add_and_set_default() {
    let bench = start_bench().await;
    let account = bench.add_default_account().await;
    let create = [
        "identity",
        "add",
        &account,
        "--address",
        "alias@folders.test",
        "--display-name",
        "Alias",
        "--json",
    ];
    let created = bench.run("", &create).await;
    assert_eq!(created.code, 0, "{}", created.stderr);
    let identity = parse_json(&created)["data"]["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let promoted = bench
        .run("", &["identity", "set-default", &identity, "--json"])
        .await;
    assert_eq!(promoted.code, 0, "{}", promoted.stderr);
    let listed = bench.run("", &["identity", "list", &account]).await;
    assert!(listed.stdout.contains("alias@folders.test"));
}

/// Une opération inconnue est « introuvable » (11), enveloppe d'erreur en JSON
#[tokio::test]
async fn unknown_operation_is_not_found() {
    let bench = start_bench().await;
    let result = bench.run("", &["operation", "get", "nope", "--json"]).await;
    assert_eq!(result.code, 11);
    assert_eq!(parse_json(&result)["error"]["category"], "not_found");
}
