#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Commandes `mira queue …` et attente `--wait` : file locale, annulation, relance, délai dépassé.

#[allow(dead_code)]
mod cli_support;

use cli_support::full_bench::FullBench;

/// `queue list --state` : filtre par état ; sans option, toutes les actions, dans l'ordre de la file
#[tokio::test]
async fn queue_list_filters_by_state() {
    let (bench, account) = FullBench::start_seeded().await;
    let first = bench
        .run_ok(&["action", "apply", "flag", "--message", "m1"])
        .await;
    let id = first["data"]["data"][0].as_str().unwrap().to_string();
    bench.run_ok(&["action", "undo", &id]).await;
    bench
        .run_ok(&["action", "apply", "trash", "--message", "m2"])
        .await;
    let all = bench.queued_actions(&account).await;
    let pending = bench
        .run_ok(&["queue", "list", &account, "--state", "pending"])
        .await;
    let both = bench
        .run_ok(&[
            "queue",
            "list",
            &account,
            "--state",
            "pending",
            "--state",
            "cancelled",
        ])
        .await;
    assert_eq!(all.len(), 2);
    assert_eq!(pending["data"]["data"].as_array().unwrap().len(), 1);
    assert_eq!(pending["data"]["data"][0]["kind"]["type"], "trash");
    assert_eq!(both["data"]["data"].as_array().unwrap().len(), 2);
}

/// `queue list` en sortie humaine : une ligne par action avec nature et état
#[tokio::test]
async fn queue_list_human_output_is_a_table() {
    let (bench, account) = FullBench::start_seeded().await;
    bench
        .run_ok(&["action", "apply", "archive", "--message", "m1"])
        .await;
    let table = bench.run(&["queue", "list", &account]).await;
    assert_eq!(table.code, 0, "{}", table.stderr);
    let lines: Vec<&str> = table.stdout.lines().collect();
    assert!(
        lines[0].starts_with("ACTION") && lines[0].contains("NATURE"),
        "{}",
        lines[0]
    );
    assert!(
        lines[1].contains("archive") && lines[1].contains("pending"),
        "{}",
        lines[1]
    );
}

/// `queue cancel` annule une action en attente ; ensuite code 12 ; action inconnue code 11
#[tokio::test]
async fn queue_cancel_then_retry_of_a_non_failed_action_is_refused() {
    let (bench, account) = FullBench::start_seeded().await;
    let applied = bench
        .run_ok(&["action", "apply", "flag", "--message", "m1"])
        .await;
    let id = applied["data"]["data"][0].as_str().unwrap().to_string();
    let retry = bench.run(&["queue", "retry", &id]).await;
    assert_eq!(retry.code, 12, "{}", retry.stderr);
    bench.run_ok(&["queue", "cancel", &id]).await;
    assert_eq!(
        bench.queued_actions(&account).await[0]["state"],
        "cancelled"
    );
    assert_eq!(bench.run(&["queue", "cancel", &id]).await.code, 12);
    assert_eq!(bench.run(&["queue", "cancel", "inconnue"]).await.code, 11);
}

/// Refus du serveur : `--wait` sort en code 16 et l'action reste échouée ; `queue retry --wait` la mène à bien
#[tokio::test]
async fn wait_reports_a_server_refusal_and_retry_succeeds() {
    let (bench, account) = FullBench::start_seeded().await;
    let pump = bench.pump_clock();
    bench
        .store
        .fail_next("UID MOVE", "NO [OVERQUOTA] mailbox full");
    let line = [
        "action",
        "apply",
        "move",
        "--message",
        "m1",
        "--to",
        "box-arch",
        "--wait",
    ];
    let refused = bench.run(&line).await;
    assert_eq!(refused.code, 16, "{} {}", refused.stdout, refused.stderr);
    assert!(refused.stderr.contains("server.rejected"));
    let failed = bench.queued_actions(&account).await;
    assert_eq!(failed[0]["state"], "failed");
    let id = failed[0]["id"].as_str().unwrap().to_string();
    bench.run_ok(&["queue", "retry", &id, "--wait"]).await;
    pump.abort();
    assert_eq!(
        bench.store.messages("alice@folders.test", "Archive").len(),
        1
    );
}

/// Serveur injoignable : `--wait --timeout 1` sort en code 13 (réseau), l'action reste en attente
#[tokio::test]
async fn wait_times_out_when_the_server_is_unreachable() {
    let (bench, account) = FullBench::start_seeded().await;
    bench.imap.shutdown();
    let pump = bench.pump_clock();
    let line = [
        "action",
        "apply",
        "mark-read",
        "--message",
        "m1",
        "--wait",
        "--timeout",
        "1",
    ];
    let result = bench.run(&line).await;
    pump.abort();
    assert_eq!(result.code, 13, "{} {}", result.stdout, result.stderr);
    assert!(result.stderr.contains("network.timeout"));
    assert_eq!(bench.queued_actions(&account).await[0]["state"], "pending");
}

/// `queue list` d'un compte inconnu : code 11 ; état inconnu : code 10
#[tokio::test]
async fn queue_list_errors_use_the_contract_exit_codes() {
    let (bench, account) = FullBench::start_seeded().await;
    let unknown = bench.run(&["queue", "list", "inconnu"]).await;
    assert_eq!(unknown.code, 11, "{}", unknown.stderr);
    let state = bench
        .run(&["queue", "list", &account, "--state", "bizarre"])
        .await;
    assert_eq!(state.code, 10);
}
