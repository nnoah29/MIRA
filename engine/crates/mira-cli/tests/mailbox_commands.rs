#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Commandes `mira mailbox …` : lecture hors ligne du miroir garni, écritures mises en file.

#[allow(dead_code)]
mod cli_support;

use cli_support::full_bench::FullBench;

/// `mailbox list` : boîtes et compteurs lus dans le miroir, serveurs arrêtés (invariant 27)
#[tokio::test]
async fn mailbox_list_reads_the_mirror_while_offline() {
    let (bench, account) = FullBench::start_seeded().await;
    bench.imap.shutdown();
    let data = bench.run_ok(&["mailbox", "list", &account]).await;
    assert_eq!(data["type"], "mailboxes");
    let boxes = data["data"].as_array().unwrap();
    let inbox = boxes.iter().find(|b| b["id"] == "box-in").unwrap();
    assert_eq!(
        (inbox["total"].as_u64(), inbox["unread"].as_u64()),
        (Some(5), Some(3))
    );
    assert!(
        boxes
            .iter()
            .any(|b| b["id"] == "box-arch" && b["role"] == "archive")
    );
}

/// `mailbox list` en sortie humaine : un tableau avec en-têtes et une ligne par boîte
#[tokio::test]
async fn mailbox_list_human_output_is_a_table() {
    let (bench, account) = FullBench::start_seeded().await;
    let result = bench.run(&["mailbox", "list", &account]).await;
    assert_eq!(result.code, 0, "{}", result.stderr);
    let first = result.stdout.lines().next().unwrap();
    assert!(first.starts_with("ID"), "{first}");
    assert!(result.stdout.contains("INBOX"));
    assert!(
        result
            .stdout
            .lines()
            .any(|l| l.contains("box-trash") && l.contains("trash"))
    );
}

/// `mailbox create` hors ligne : l'identifiant d'action revient tout de suite, la file le garde
#[tokio::test]
async fn mailbox_create_returns_an_action_while_offline() {
    let (bench, account) = FullBench::start_seeded().await;
    bench.imap.shutdown();
    let created = bench
        .run_ok(&[
            "mailbox", "create", &account, "Projets", "--parent", "box-in",
        ])
        .await;
    let action = created["data"]["data"].as_str().unwrap().to_string();
    let queue = bench.run_ok(&["queue", "list", &account]).await;
    let queued = queue["data"]["data"].as_array().unwrap();
    assert_eq!(queued.len(), 1);
    assert_eq!(queued[0]["id"], action.as_str());
    assert_eq!(queued[0]["state"], "pending");
    assert_eq!(queued[0]["kind"]["type"], "create_mailbox");
}

/// `mailbox create --key` : la même clé ne crée qu'une action
#[tokio::test]
async fn mailbox_create_with_the_same_key_is_idempotent() {
    let (bench, account) = FullBench::start_seeded().await;
    let arguments = ["mailbox", "create", &account, "Projets", "--key", "cle-1"];
    let first = bench.run_ok(&arguments).await;
    let second = bench.run_ok(&arguments).await;
    assert_eq!(first["data"], second["data"]);
    let queue = bench.run_ok(&["queue", "list", &account]).await;
    assert_eq!(queue["data"]["data"].as_array().unwrap().len(), 1);
}

/// Renommer ou supprimer une boîte système est refusé (état incompatible, code de sortie 12)
#[tokio::test]
async fn system_mailbox_rename_and_delete_exit_with_code_12() {
    let (bench, _account) = FullBench::start_seeded().await;
    let rename = bench
        .run(&["mailbox", "rename", "box-in", "Autre", "--json"])
        .await;
    assert_eq!(rename.code, 12, "{}", rename.stdout);
    assert!(rename.stdout.contains("incompatible_state.system_mailbox"));
    let delete = bench.run(&["mailbox", "delete", "box-trash"]).await;
    assert_eq!(delete.code, 12);
    assert!(delete.stderr.contains("incompatible_state.system_mailbox"));
}

/// `mailbox set-synced` change les boîtes synchronisées ; une boîte inconnue donne le code 10
#[tokio::test]
async fn mailbox_set_synced_updates_the_sync_flags() {
    let (bench, account) = FullBench::start_seeded().await;
    bench
        .run_ok(&["mailbox", "set-synced", &account, "box-in", "box-arch"])
        .await;
    let data = bench.run_ok(&["mailbox", "list", &account]).await;
    let synced = |id: &str| {
        let boxes = data["data"].as_array().unwrap();
        boxes.iter().find(|b| b["id"] == id).unwrap()["sync_enabled"].clone()
    };
    assert_eq!(
        (synced("box-in"), synced("box-arch"), synced("box-trash")),
        (true.into(), true.into(), false.into())
    );
    let unknown = bench
        .run(&["mailbox", "set-synced", &account, "nope"])
        .await;
    assert_eq!(unknown.code, 10, "{}", unknown.stderr);
}
