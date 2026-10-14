#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `--json` des commandes de lecture et d'écriture : la réponse du contrat telle quelle, sans secret.

#[allow(dead_code)]
mod cli_support;

use cli_support::full_bench::{FullBench, PASSWORD};
use cli_support::parse_json;
use cli_support::run_cli;
use mira_assembly::api::Response;

/// Chaque réponse `--json` se relit comme une `Response` du contrat et se resérialise à l'identique
#[tokio::test]
async fn json_output_is_the_contract_response_itself() {
    let (bench, account) = FullBench::start_seeded().await;
    let lines: [&[&str]; 8] = [
        &["mailbox", "list", &account],
        &["message", "list", "--mailbox", "box-in"],
        &["message", "get", "m1"],
        &["message", "body", "m1"],
        &["thread", "get", "t-r1"],
        &["draft", "list", &account],
        &["queue", "list", &account],
        &["maintenance", "stats"],
    ];
    for line in lines {
        let data = bench.run_ok(line).await;
        let response: Response = serde_json::from_value(data.clone()).unwrap();
        assert_eq!(serde_json::to_value(&response).unwrap(), data, "{line:?}");
    }
}

/// Une erreur de lecture : enveloppe `ok: false`, catégorie, code stable, contexte et code de sortie
#[tokio::test]
async fn read_error_envelope_carries_category_code_and_context() {
    let (bench, _account) = FullBench::start_seeded().await;
    let result = bench.run(&["message", "body", "m2", "--json"]).await;
    assert_eq!(result.code, 12);
    let document = parse_json(&result);
    assert_eq!(document["ok"], false);
    assert_eq!(document["contract_version"]["major"], 1);
    let error = &document["error"];
    assert_eq!(error["category"], "incompatible_state");
    assert_eq!(error["code"], "incompatible_state.body_unavailable_offline");
    assert_eq!(error["retryable"], false);
    assert_eq!(error["context"]["message"], "m2");
    assert!(result.stderr.is_empty());
}

/// Aucune sortie, humaine ou JSON, ne contient le mot de passe du compte
#[tokio::test]
async fn no_output_contains_the_account_password() {
    let (bench, account) = FullBench::start_seeded().await;
    let lines: [&[&str]; 7] = [
        &["engine", "status"],
        &["account", "list"],
        &["mailbox", "list", &account],
        &["message", "list", "--mailbox", "box-in"],
        &["queue", "list", &account],
        &["events"],
        &["message", "fetch", "m2"],
    ];
    for line in lines {
        let plain = bench.run(line).await;
        let json = bench.run(&[line, &["--json"]].concat()).await;
        for text in [&plain.stdout, &plain.stderr, &json.stdout, &json.stderr] {
            assert!(!text.contains(PASSWORD), "{line:?}");
        }
    }
}

/// L'aide principale liste les nouveaux domaines ; une sous-commande sans verbe est une erreur d'usage
#[test]
fn help_lists_the_new_domains() {
    let help = run_cli(&["--help"]);
    assert_eq!(help.code, 0);
    for domain in [
        "mailbox",
        "message",
        "thread",
        "search",
        "maintenance",
        "action",
        "draft",
        "queue",
    ] {
        assert!(help.stdout.contains(domain), "{domain}");
    }
    let verbs = run_cli(&["draft", "--help"]);
    for verb in ["create", "send", "cancel-send", "outbox", "resolve-send"] {
        assert!(verbs.stdout.contains(verb), "{verb}");
    }
    assert_eq!(run_cli(&["message"]).code, 10);
}
