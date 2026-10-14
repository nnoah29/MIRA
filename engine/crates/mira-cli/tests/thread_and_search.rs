#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Commandes `mira thread …` et `mira search local` : lectures locales, hors ligne.

#[allow(dead_code)]
mod cli_support;

use cli_support::full_bench::FullBench;
use cli_support::json_views::page_ids;

/// `thread list` : les fils de la boîte, le plus récent d'abord, avec leurs compteurs
#[tokio::test]
async fn thread_list_shows_threads_of_a_mailbox() {
    let (bench, _account) = FullBench::start_seeded().await;
    bench.imap.shutdown();
    let data = bench.run_ok(&["thread", "list", "box-in"]).await;
    assert_eq!(page_ids(&data), ["t-r2", "t-r1"]);
    let human = bench.run(&["thread", "list", "box-in"]).await;
    let first = human.stdout.lines().next().unwrap();
    assert!(
        first.starts_with("ID") && first.contains("NON LUS/TOTAL"),
        "{first}"
    );
    assert!(
        human
            .stdout
            .lines()
            .any(|l| l.starts_with("t-r1") && l.contains("2/2"))
    );
}

/// `thread get` : un fil et ses messages ; un fil inconnu sort avec le code 11
#[tokio::test]
async fn thread_get_returns_messages_and_unknown_thread_exits_11() {
    let (bench, _account) = FullBench::start_seeded().await;
    let data = bench.run_ok(&["thread", "get", "t-r1"]).await;
    let messages = data["data"]["messages"].as_array().unwrap();
    let ids: Vec<&str> = messages.iter().map(|m| m["id"].as_str().unwrap()).collect();
    assert_eq!(ids, ["m1", "m2"]);
    let human = bench.run(&["thread", "get", "t-r1"]).await;
    assert!(human.stdout.starts_with("Fil t-r1"), "{}", human.stdout);
    let unknown = bench.run(&["thread", "get", "absent"]).await;
    assert_eq!(unknown.code, 11);
    assert!(unknown.stderr.contains("not_found.thread"));
}

/// `search local` : plusieurs mots (ET), filtre d'expéditeur et de boîte, serveur arrêté
#[tokio::test]
async fn search_local_combines_words_and_filters_offline() {
    let (bench, _account) = FullBench::start_seeded().await;
    bench.imap.shutdown();
    let found = bench
        .run_ok(&["search", "local", "bonjour", "licorne"])
        .await;
    assert_eq!(page_ids(&found), ["m1"]);
    let by_sender = bench
        .run_ok(&["search", "local", "licorne", "--from", "carla"])
        .await;
    assert!(page_ids(&by_sender).is_empty());
    let by_box = bench
        .run_ok(&["search", "local", "licorne", "--mailbox", "box-arch"])
        .await;
    assert!(page_ids(&by_box).is_empty());
    let in_box = bench
        .run_ok(&["search", "local", "licorne", "--mailbox", "box-in"])
        .await;
    assert_eq!(page_ids(&in_box), ["m1"]);
}

/// `search local --limit` : la page suivante se demande avec le curseur rendu
#[tokio::test]
async fn search_local_paginates_with_limit_and_cursor() {
    let (bench, _account) = FullBench::start_seeded().await;
    let first = bench
        .run_ok(&["search", "local", "sujet", "--limit", "1"])
        .await;
    assert_eq!(page_ids(&first), ["m3"]);
    let cursor = first["data"]["next_cursor"].as_str().unwrap().to_string();
    let args = [
        "search", "local", "sujet", "--limit", "1", "--cursor", &cursor,
    ];
    let second = bench.run_ok(&args).await;
    assert_eq!(page_ids(&second), ["m1"]);
    assert!(second["data"]["next_cursor"].is_null());
}

/// `search local` sans mot ou avec une date invalide : entrée invalide, code 10
#[tokio::test]
async fn search_local_rejects_missing_words_and_bad_dates() {
    let (bench, _account) = FullBench::start_seeded().await;
    let no_words = bench.run(&["search", "local"]).await;
    assert_eq!(no_words.code, 10);
    let bad_date = bench
        .run(&["search", "local", "x", "--since", "demain"])
        .await;
    assert_eq!(bad_date.code, 10, "{}", bad_date.stderr);
    assert!(bad_date.stderr.contains("instant invalide"));
}
