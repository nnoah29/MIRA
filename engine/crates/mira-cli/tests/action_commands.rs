#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Commandes `mira action …` : lots par liste ou par requête, idempotence, annulation, attente du rejeu.

#[allow(dead_code)]
mod cli_support;

use cli_support::full_bench::{FullBench, USER};
use cli_support::json_views::words;

/// Hors ligne, `action apply` rend tout de suite l'identifiant d'action et l'effet local est visible
#[tokio::test]
async fn apply_returns_an_action_id_and_applies_locally_while_offline() {
    let (bench, account) = FullBench::start_seeded().await;
    bench.imap.shutdown();
    let applied = bench
        .run_ok(&[
            "action",
            "apply",
            "mark-read",
            "--message",
            "m1",
            "--message",
            "m2",
        ])
        .await;
    assert_eq!(applied["data"]["type"], "actions");
    assert_eq!(applied["data"]["data"].as_array().unwrap().len(), 1);
    let seen = bench.run_ok(&["message", "get", "m1"]).await;
    assert_eq!(seen["data"]["flags"]["seen"], true);
    let actions = bench.queued_actions(&account).await;
    assert_eq!(actions[0]["state"], "pending");
    assert_eq!(
        actions[0]["kind"],
        serde_json::json!({"type": "mark_read", "read": true})
    );
    assert_eq!(actions[0]["targets"], serde_json::json!(["m1", "m2"]));
}

/// Sélection par requête : la vue et les filtres choisissent les messages au moment de la soumission
#[tokio::test]
async fn apply_by_query_selects_the_matching_messages() {
    let (bench, account) = FullBench::start_seeded().await;
    bench
        .run_ok(&["action", "apply", "flag", "--mailbox", "box-in", "--unread"])
        .await;
    let actions = bench.queued_actions(&account).await;
    let mut targets: Vec<String> = actions[0]["targets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t.as_str().unwrap().to_string())
        .collect();
    targets.sort();
    assert_eq!(targets, ["m1", "m2", "m5"]);
    assert_eq!(actions[0]["kind"]["type"], "mark_flagged");
}

/// La même clé d'idempotence ne crée qu'une action ; sans clé, chaque appel en crée une
#[tokio::test]
async fn apply_with_the_same_key_creates_one_action() {
    let (bench, account) = FullBench::start_seeded().await;
    let line = [
        "action",
        "apply",
        "archive",
        "--message",
        "m3",
        "--key",
        "cle-7",
    ];
    let first = bench.run_ok(&line).await;
    let second = bench.run_ok(&line).await;
    assert_eq!(first["data"], second["data"]);
    assert_eq!(bench.queued_actions(&account).await.len(), 1);
    bench
        .run_ok(&["action", "apply", "trash", "--message", "m4"])
        .await;
    assert_eq!(bench.queued_actions(&account).await.len(), 2);
}

/// `action undo` annule une action en attente et restaure l'état local ; une seconde fois : code 12
#[tokio::test]
async fn undo_cancels_a_pending_action_once() {
    let (bench, account) = FullBench::start_seeded().await;
    let applied = bench
        .run_ok(&["action", "apply", "mark-read", "--message", "m1"])
        .await;
    let action = applied["data"]["data"][0].as_str().unwrap().to_string();
    bench.run_ok(&["action", "undo", &action]).await;
    assert_eq!(
        bench.queued_actions(&account).await[0]["state"],
        "cancelled"
    );
    let unread = bench.run_ok(&["message", "get", "m1"]).await;
    assert_eq!(unread["data"]["flags"]["seen"], false);
    let again = bench.run(&["action", "undo", &action]).await;
    assert_eq!(again.code, 12, "{}", again.stderr);
    let unknown = bench.run(&["action", "undo", "inconnue"]).await;
    assert_eq!(unknown.code, 11);
}

/// Sélections ou natures incohérentes : refus avant tout appel au moteur, code 10
#[tokio::test]
async fn inconsistent_selections_exit_with_code_10() {
    let (bench, _account) = FullBench::start_seeded().await;
    let cases = [
        "action apply move --message m1",
        "action apply archive --message m1 --to box-arch",
        "action apply mark-read",
        "action apply mark-read --message m1 --mailbox box-in",
        "action apply frobnicate --message m1",
    ];
    for line in cases {
        let result = bench.run(&words(line)).await;
        assert_eq!(result.code, 10, "{line:?} : {}", result.stderr);
        assert!(result.stdout.is_empty());
    }
}

/// `--wait` : la commande rend la main une fois l'action rejouée, le serveur a le drapeau lu
#[tokio::test]
async fn apply_wait_returns_after_the_server_has_the_change() {
    let (bench, _account) = FullBench::start_seeded().await;
    let pump = bench.pump_clock();
    let applied = bench
        .run_ok(&["action", "apply", "mark-read", "--message", "m1", "--wait"])
        .await;
    pump.abort();
    assert_eq!(applied["data"]["type"], "actions");
    let views = bench.store.messages(USER, "INBOX");
    let first = views.iter().find(|view| view.uid == 1).unwrap();
    assert!(
        first
            .flags
            .iter()
            .any(|flag| flag.eq_ignore_ascii_case("\\Seen"))
    );
}
