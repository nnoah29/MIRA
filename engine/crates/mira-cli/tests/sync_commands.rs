#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Commandes `mira sync …` en processus contre le faux serveur IMAP : première synchro, état, pause.

#[allow(dead_code)]
mod cli_support;

use cli_support::full_bench::{FullBench, PASSWORD};

/// Bench dont le serveur porte trois courriers en réception et dont le compte est ajouté sans synchro.
async fn bench_with_inbox() -> (FullBench, String) {
    let bench = FullBench::start().await;
    for subject in ["Alpha", "Beta", "Gamma"] {
        bench.deliver_subject("INBOX", subject);
    }
    let account = bench.add_account().await;
    (bench, account)
}

/// `sync start --wait` attend la première synchro : le miroir se lit ensuite sans serveur
#[tokio::test]
async fn sync_start_wait_fills_the_mirror_readable_offline() {
    let (bench, account) = bench_with_inbox().await;
    assert!(
        bench.run_ok(&["mailbox", "list", &account]).await["data"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let data = bench.run_sync_ok(&["sync", "start", "--wait"]).await;
    assert_eq!(data["type"], "operation");
    assert_eq!(data["data"]["kind"], "sync");
    assert_eq!(data["data"]["status"]["type"], "finished");
    bench.imap.shutdown();
    let inbox = bench.mailbox_named(&account, "INBOX").await;
    let mut subjects = bench.subjects_of(&inbox).await;
    subjects.sort();
    assert_eq!(subjects, ["Alpha", "Beta", "Gamma"]);
}

/// `sync status` rend une ligne par compte : connecté, au repos, dernier succès renseigné
#[tokio::test]
async fn sync_status_reports_last_success_per_account() {
    let (bench, account) = bench_with_inbox().await;
    let before = bench.run_ok(&["sync", "status"]).await;
    assert!(before["data"]["data"][0]["last_synced_at"].is_null());
    bench
        .run_sync_ok(&["sync", "start", "--account", &account])
        .await;
    let after = bench.run_ok(&["sync", "status"]).await;
    let status = &after["data"]["data"][0];
    assert_eq!(status["account"], account.as_str());
    assert_eq!(status["state"]["type"], "connected");
    assert_eq!(status["syncing"], false);
    assert!(status["last_synced_at"].is_i64() || status["last_synced_at"].is_u64());
    assert_eq!(status["pending_actions"], 0);
}

/// Sortie humaine de `sync status` : tableau avec en-têtes et identifiant du compte
#[tokio::test]
async fn sync_status_human_output_is_a_table() {
    let (bench, account) = bench_with_inbox().await;
    bench.run_sync_ok(&["sync", "start"]).await;
    let result = bench.run(&["sync", "status"]).await;
    assert_eq!(result.code, 0, "{}", result.stderr);
    let mut lines = result.stdout.lines();
    let header = lines.next().unwrap();
    assert!(header.starts_with("COMPTE"), "{header}");
    assert!(header.contains("DERNIER SUCCÈS"));
    let row = lines.next().unwrap();
    assert!(row.contains(&account));
    assert!(row.contains("connected"));
    assert!(row.contains("au repos"));
}

/// `sync pause` met le compte en pause ; la synchro de ce compte est ensuite refusée (code 12)
#[tokio::test]
async fn sync_pause_stops_the_account_and_refuses_new_requests() {
    let (bench, account) = bench_with_inbox().await;
    let paused = bench
        .run_ok(&["sync", "pause", "--account", &account])
        .await;
    assert_eq!(paused["data"]["type"], "paused");
    assert_eq!(paused["data"]["data"]["state"]["type"], "paused");
    let refused = bench
        .run_sync(&["sync", "start", "--account", &account])
        .await;
    assert_eq!(refused.code, 12);
    assert!(
        refused.stderr.contains("account_paused"),
        "{}",
        refused.stderr
    );
    let connections = bench.imap.connection_count();
    let all = bench.run_sync_ok(&["sync", "start"]).await;
    assert_eq!(all["data"]["status"]["type"], "finished");
    assert_eq!(bench.imap.connection_count(), connections);
}

/// Aucune sortie de synchro ne contient le mot de passe du compte
#[tokio::test]
async fn sync_output_never_contains_the_password() {
    let (bench, account) = bench_with_inbox().await;
    let started = bench
        .run_sync(&["sync", "start", "--account", &account])
        .await;
    let status = bench.run(&["sync", "status", "--json"]).await;
    for text in [
        &started.stdout,
        &started.stderr,
        &status.stdout,
        &status.stderr,
    ] {
        assert!(!text.contains(PASSWORD));
    }
    assert_eq!(started.code, 0, "{}", started.stderr);
}
