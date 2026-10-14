#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Commandes `mira maintenance …` : volumes, intégrité, index, cache, exports sans secret.

#[allow(dead_code)]
mod cli_support;

use cli_support::full_bench::{FullBench, PASSWORD, USER};

/// `maintenance stats` : volumes du miroir en JSON et en texte
#[tokio::test]
async fn maintenance_stats_counts_the_mirror() {
    let (bench, _account) = FullBench::start_seeded().await;
    let data = bench.run_ok(&["maintenance", "stats"]).await;
    assert_eq!(data["type"], "storage_stats");
    assert_eq!(data["data"]["messages"], 5);
    assert_eq!(data["data"]["raw_sources"], 2);
    let human = bench.run(&["maintenance", "stats"]).await;
    assert!(
        human.stdout.contains("Messages       : 5"),
        "{}",
        human.stdout
    );
}

/// `maintenance check` et `rebuild-index` : la base est intègre, la recherche survit à la reconstruction
#[tokio::test]
async fn maintenance_check_and_rebuild_index_keep_search_working() {
    let (bench, _account) = FullBench::start_seeded().await;
    let check = bench.run_ok(&["maintenance", "check"]).await;
    assert_eq!(check["data"]["ok"], true);
    assert_eq!(
        bench.run(&["maintenance", "check"]).await.stdout.trim(),
        "base intègre"
    );
    let rebuilt = bench.run_ok(&["maintenance", "rebuild-index"]).await;
    assert_eq!(rebuilt["type"], "done");
    let found = bench.run_ok(&["search", "local", "licorne"]).await;
    assert_eq!(found["data"]["items"].as_array().unwrap().len(), 1);
}

/// `maintenance purge-attachments` : les pièces jointes téléchargées redeviennent à télécharger
#[tokio::test]
async fn maintenance_purge_attachments_empties_the_cache() {
    let (bench, _account) = FullBench::start_seeded().await;
    bench.run_ok(&["message", "fetch", "m4"]).await;
    let list = bench.run_ok(&["message", "attachments", "m4"]).await;
    let attachment = list["data"][0]["id"].as_str().unwrap().to_string();
    let folder = tempfile::tempdir().unwrap();
    let file = folder.path().join("data.bin");
    let saved = [
        "message",
        "save-attachment",
        &attachment,
        file.to_str().unwrap(),
    ];
    bench.run_ok(&saved).await;
    let before = bench.run_ok(&["maintenance", "stats"]).await;
    bench.run_ok(&["maintenance", "purge-attachments"]).await;
    let after = bench.run_ok(&["maintenance", "stats"]).await;
    assert_eq!(before["data"]["attachments_cached"], 1);
    assert_eq!(after["data"]["attachments"], 1);
    assert_eq!(after["data"]["attachments_cached"], 0);
}

/// `maintenance export-diagnostic` : fichier écrit, sans adresse ni secret
#[tokio::test]
async fn export_diagnostic_writes_a_file_without_secret_or_address() {
    let (bench, _account) = FullBench::start_seeded().await;
    let target = tempfile::tempdir().unwrap();
    let path = target.path().join("diagnostic.json");
    let destination = path.to_str().unwrap();
    let report = bench
        .run_ok(&["maintenance", "export-diagnostic", destination])
        .await;
    assert_eq!(report["data"]["path"], destination);
    let content = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        report["data"]["bytes"].as_u64().unwrap(),
        content.len() as u64
    );
    assert!(!content.contains(PASSWORD) && !content.contains(USER));
    let human = bench
        .run(&["maintenance", "export-diagnostic", destination])
        .await;
    assert!(human.stdout.starts_with("export écrit : "));
}

/// `maintenance export-profile` : configuration et intentions, jamais de secret ; destination absente, code 17
#[tokio::test]
async fn export_profile_has_no_secret_and_bad_destination_exits_17() {
    let (bench, _account) = FullBench::start_seeded().await;
    let target = tempfile::tempdir().unwrap();
    let path = target.path().join("profil.json");
    let destination = path.to_str().unwrap();
    bench
        .run_ok(&["maintenance", "export-profile", destination])
        .await;
    let exported: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(exported["format_version"], 1);
    let text = exported.to_string();
    assert!(!text.contains(PASSWORD) && text.contains(USER));
    let missing = target.path().join("absent").join("profil.json");
    let failed = bench
        .run(&["maintenance", "export-profile", missing.to_str().unwrap()])
        .await;
    assert_eq!(failed.code, 17, "{}", failed.stderr);
}
