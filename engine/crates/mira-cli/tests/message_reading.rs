#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Commandes `mira message …` de lecture : listes paginées et filtrées, détail, corps, source, HTML.

#[allow(dead_code)]
mod cli_support;

use cli_support::full_bench::FullBench;
use cli_support::json_views::{page_ids as ids_of, with_args};

/// Pagination : pages de deux messages, du plus récent au plus ancien, reprise par `--cursor`
#[tokio::test]
async fn message_list_pages_follow_the_cursor() {
    let (bench, _account) = FullBench::start_seeded().await;
    bench.imap.shutdown();
    let list = ["message", "list", "--mailbox", "box-in", "--limit", "2"];
    let first = bench.run_ok(&list).await;
    assert_eq!(ids_of(&first), ["m5", "m4"]);
    let cursor = first["data"]["next_cursor"].as_str().unwrap().to_string();
    let second = bench
        .run_ok(&with_args(&list, &["--cursor", &cursor]))
        .await;
    assert_eq!(ids_of(&second), ["m3", "m2"]);
    let cursor = second["data"]["next_cursor"].as_str().unwrap().to_string();
    let third = bench
        .run_ok(&with_args(&list, &["--cursor", &cursor]))
        .await;
    assert_eq!(ids_of(&third), ["m1"]);
    assert!(third["data"]["next_cursor"].is_null());
}

/// Filtres : non lus, suivis, expéditeur, intervalle de dates (millisecondes ou date UTC)
#[tokio::test]
async fn message_list_filters_narrow_the_page() {
    let (bench, _account) = FullBench::start_seeded().await;
    let base = ["message", "list", "--mailbox", "box-in"];
    let with = |extra: &[&'static str]| with_args(&base, extra);
    let unread = bench.run_ok(&with(&["--unread"])).await;
    assert_eq!(ids_of(&unread), ["m5", "m2", "m1"]);
    let flagged = bench.run_ok(&with(&["--flagged"])).await;
    assert_eq!(ids_of(&flagged), ["m2"]);
    let carla = bench.run_ok(&with(&["--from", "carla"])).await;
    assert_eq!(ids_of(&carla), ["m5", "m4"]);
    let range = bench
        .run_ok(&with(&["--since", "3000", "--until", "4000"]))
        .await;
    assert_eq!(ids_of(&range), ["m4", "m3"]);
    let dated = with(&[
        "--since",
        "1970-01-01T00:00:03",
        "--until",
        "1970-01-01T00:00:04",
    ]);
    assert_eq!(ids_of(&bench.run_ok(&dated).await), ["m4", "m3"]);
}

/// Sortie humaine : tableau des messages et rappel de la page suivante
#[tokio::test]
async fn message_list_human_output_shows_table_and_next_page_hint() {
    let (bench, _account) = FullBench::start_seeded().await;
    let result = bench
        .run(&["message", "list", "--mailbox", "box-in", "--limit", "2"])
        .await;
    assert_eq!(result.code, 0, "{}", result.stderr);
    let lines: Vec<&str> = result.stdout.lines().collect();
    assert!(
        lines[0].starts_with("ID") && lines[0].contains("SUJET"),
        "{}",
        lines[0]
    );
    assert!(
        lines[1].starts_with("m5") && lines[1].contains("Carla"),
        "{}",
        lines[1]
    );
    assert!(lines[1].contains("non-lu"));
    assert!(lines.last().unwrap().starts_with("suite : --cursor "));
}

/// Détail, corps, source brute et HTML assaini d'un message téléchargé, serveur arrêté
#[tokio::test]
async fn message_get_body_raw_and_safe_html_work_offline() {
    let (bench, _account) = FullBench::start_seeded().await;
    bench.imap.shutdown();
    let got = bench.run_ok(&["message", "get", "m1"]).await;
    assert_eq!(got["data"]["subject"], "Sujet m1");
    let body = bench.run_ok(&["message", "body", "m1"]).await;
    assert_eq!(body["data"]["text"], "Bonjour monde licorne");
    let raw = bench.run_ok(&["message", "raw", "m1"]).await;
    assert_eq!(raw["data"]["size"], 21);
    let html = bench.run_ok(&["message", "safe-html", "m1"]).await;
    assert!(
        html["data"]["html"]
            .as_str()
            .unwrap()
            .contains("<b>monde</b>")
    );
    let text = bench.run(&["message", "body", "m1"]).await;
    assert!(text.stdout.starts_with("Bonjour monde licorne"));
    let source = bench.run(&["message", "raw", "m1"]).await;
    assert_eq!(source.stdout.trim_end(), "Bonjour monde licorne");
}

/// Un corps jamais téléchargé donne le code 12 hors ligne ; un message inconnu le code 11
#[tokio::test]
async fn message_body_errors_use_the_contract_exit_codes() {
    let (bench, _account) = FullBench::start_seeded().await;
    bench.imap.shutdown();
    let headers_only = bench.run(&["message", "body", "m2", "--json"]).await;
    assert_eq!(headers_only.code, 12, "{}", headers_only.stdout);
    assert!(
        headers_only
            .stdout
            .contains("incompatible_state.body_unavailable_offline")
    );
    let unknown = bench.run(&["message", "get", "inconnu"]).await;
    assert_eq!(unknown.code, 11);
    assert!(unknown.stderr.contains("not_found.message"));
}

/// Autoriser le contenu distant d'un expéditeur ; sans cible, entrée invalide (code 10)
#[tokio::test]
async fn message_allow_remote_needs_a_target() {
    let (bench, _account) = FullBench::start_seeded().await;
    let allowed = bench
        .run_ok(&["message", "allow-remote", "--sender", "bob@example.org"])
        .await;
    assert_eq!(allowed["type"], "done");
    let missing = bench.run(&["message", "allow-remote"]).await;
    assert_eq!(missing.code, 10);
    let listing = bench.run(&["message", "list"]).await;
    assert_eq!(listing.code, 10);
    assert!(listing.stderr.contains("--mailbox ou --view"));
}
