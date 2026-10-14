#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Commandes réseau explicites : téléchargement d'un corps, d'une pièce jointe, recherche serveur.

#[allow(dead_code)]
mod cli_support;

use cli_support::full_bench::FullBench;
use cli_support::raw_mail::ATTACHMENT_SIZE;

/// `message fetch` attend la fin du téléchargement ; le corps se lit ensuite hors ligne
#[tokio::test]
async fn message_fetch_downloads_the_body_then_reads_offline() {
    let (bench, _account) = FullBench::start_seeded().await;
    let fetched = bench.run_ok(&["message", "fetch", "m2"]).await;
    assert_eq!(fetched["type"], "operation");
    assert_eq!(fetched["data"]["kind"], "fetch_message_body");
    assert_eq!(fetched["data"]["status"]["type"], "finished");
    bench.imap.shutdown();
    let body = bench.run_ok(&["message", "body", "m2"]).await;
    assert_eq!(body["data"]["text"].as_str().unwrap().trim(), "rien");
}

/// `message fetch` serveur arrêté : l'échec de l'opération sort avec le code réseau 13
#[tokio::test]
async fn message_fetch_offline_exits_with_network_code() {
    let (bench, _account) = FullBench::start_seeded().await;
    bench.imap.shutdown();
    let result = bench.run(&["message", "fetch", "m2", "--json"]).await;
    assert_eq!(result.code, 13, "{} {}", result.stdout, result.stderr);
    assert!(result.stdout.contains("\"code\":\"network."));
}

/// `message save-attachment` écrit le fichier, de façon atomique, et attend la fin
#[tokio::test]
async fn message_save_attachment_writes_the_file() {
    let (bench, _account) = FullBench::start_seeded().await;
    bench.run_ok(&["message", "fetch", "m4"]).await;
    let list = bench.run_ok(&["message", "attachments", "m4"]).await;
    let attachment = list["data"][0]["id"].as_str().unwrap().to_string();
    assert_eq!(list["data"][0]["filename"], "data.bin");
    let target = tempfile::tempdir().unwrap();
    let destination = target.path().join("out.bin");
    let path = destination.to_str().unwrap();
    let saved = bench
        .run_ok(&["message", "save-attachment", &attachment, path])
        .await;
    assert_eq!(saved["data"]["status"]["type"], "finished");
    assert_eq!(std::fs::read(&destination).unwrap().len(), ATTACHMENT_SIZE);
    let table = bench.run(&["message", "attachments", "m4"]).await;
    assert!(table.stdout.contains("data.bin") && table.stdout.contains("cached"));
}

/// `search server` attend le résultat et le rend : messages trouvés, reliés au miroir si connus
#[tokio::test]
async fn search_server_waits_and_returns_the_hits() {
    let (bench, account) = FullBench::start_seeded().await;
    let found = bench
        .run_ok(&["search", "server", &account, "licorne"])
        .await;
    assert_eq!(found["type"], "server_search");
    let hits = found["data"]["hits"].as_array().unwrap();
    let uids: Vec<u64> = hits
        .iter()
        .map(|hit| hit["uid"].as_u64().unwrap())
        .collect();
    assert_eq!(uids, [5, 1]);
    assert_eq!(hits[1]["message"], "m1");
    let human = bench.run(&["search", "server", &account, "licorne"]).await;
    assert!(human.stdout.lines().next().unwrap().starts_with("BOÎTE"));
}

/// `search server-result` ne retrouve rien dans un autre processus : opération inconnue, code 11
#[tokio::test]
async fn search_server_result_of_a_past_process_is_not_found() {
    let (bench, _account) = FullBench::start_seeded().await;
    let result = bench.run(&["search", "server-result", "id-000042"]).await;
    assert_eq!(result.code, 11, "{}", result.stderr);
    assert!(result.stderr.contains("not_found.operation"));
}
