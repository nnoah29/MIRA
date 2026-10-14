#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! S7 par le CLI : trois déplacements hors ligne, effet local immédiat, rejeu dans l'ordre au retour du réseau.

#[allow(dead_code)]
mod cli_support;

use cli_support::sa_s7_moves::start_move_world;
use mira_testkit::imap::CapabilitySet;

/// Avec MOVE : trois actions en attente hors ligne, rejouées dans l'ordre, ids stables après synchro (inv. 7, 17, 19, 28)
#[tokio::test]
async fn scenario_s7_move_three_messages_offline_then_replay_in_order_via_cli() {
    let mut fx = start_move_world(CapabilitySet::full()).await;
    let actions = fx.move_three_offline().await;
    assert_eq!(actions.len(), 3);
    assert_eq!(
        fx.queue_view().await,
        ["k0:pending", "k1:pending", "k2:pending"]
    );
    fx.reconnect_and_replay().await;
    let states = fx.queue_view().await;
    assert!(
        states.iter().all(|entry| entry.ends_with(":succeeded")),
        "{states:?}"
    );
    assert_eq!(
        fx.world.server_subjects("Archive"),
        ["Delta", "Gamma", "Beta"]
    );
    assert_eq!(fx.world.server_subjects("INBOX"), ["Alpha"]);
    let log = fx.world.bench.store.command_log();
    let at = |prefix: &str| {
        log.iter()
            .position(|line| line.starts_with(prefix))
            .unwrap()
    };
    assert!(at("UID MOVE 4") < at("UID MOVE 3") && at("UID MOVE 3") < at("UID MOVE 2"));
    assert!(!log.iter().any(|line| line.starts_with("UID COPY")));
}

/// Après la synchro qui suit le rejeu, les messages déplacés gardent leur identifiant et les compteurs sont à jour
#[tokio::test]
async fn scenario_s7_synchronization_after_replay_keeps_ids_and_updates_counters_via_cli() {
    let mut fx = start_move_world(CapabilitySet::full()).await;
    fx.move_three_offline().await;
    fx.reconnect_and_replay().await;
    fx.world.run_sync_ok(&["sync", "start"]).await;
    let in_archive = cli_support::sa_s7_moves::message_ids_in(&fx.world, &fx.archive).await;
    let mut expected: Vec<String> = fx.ids[..3].to_vec();
    let mut found = in_archive.clone();
    expected.sort();
    found.sort();
    assert_eq!(found, expected);
    let listed = fx.world.run_ok(&["mailbox", "list", &fx.account]).await;
    let total = |name: &str| {
        let boxes = listed["data"].as_array().unwrap();
        boxes.iter().find(|item| item["name"] == name).unwrap()["total"].as_u64()
    };
    assert_eq!((total("INBOX"), total("Archive")), (Some(1), Some(3)));
    let moved = fx.world.run_ok(&["message", "get", &fx.ids[0]]).await;
    assert_eq!(moved["data"]["mailboxes"], serde_json::json!([fx.archive]));
}

/// Sans MOVE : copie, marquage `\Deleted` puis expunge ciblé, même résultat observable
#[tokio::test]
async fn scenario_s7_move_without_move_extension_copies_then_expunges_via_cli() {
    let mut fx = start_move_world(CapabilitySet::without_move()).await;
    fx.move_three_offline().await;
    fx.reconnect_and_replay().await;
    let log = fx.world.bench.store.command_log();
    let count = |prefix: &str| log.iter().filter(|line| line.starts_with(prefix)).count();
    assert_eq!(count("UID MOVE"), 0);
    assert_eq!(count("UID COPY"), 3);
    assert!(count("UID EXPUNGE") >= 3);
    assert_eq!(
        fx.world.server_subjects("Archive"),
        ["Delta", "Gamma", "Beta"]
    );
    assert_eq!(fx.world.server_subjects("INBOX"), ["Alpha"]);
}

/// Hors ligne : l'effet local est immédiat, la file survit au redémarrage, le serveur n'a rien reçu (inv. 17, 28)
#[tokio::test]
async fn scenario_s7_offline_moves_are_immediate_and_persist_across_invocations_via_cli() {
    let mut fx = start_move_world(CapabilitySet::full()).await;
    let connections = fx.world.bench.imap.connection_count();
    fx.move_three_offline().await;
    assert_eq!(fx.world.subjects_in(&fx.inbox).await, ["Alpha"]);
    assert_eq!(
        fx.world.subjects_in(&fx.archive).await,
        ["Delta", "Gamma", "Beta"]
    );
    assert_eq!(fx.world.server_subjects("INBOX").len(), 4);
    assert_eq!(fx.world.bench.imap.connection_count(), connections);
    assert_eq!(fx.queue_view().await.len(), 3);
    fx.world
        .assert_no_secret_anywhere(&[fx.world.password.as_str()]);
}

/// Rejouer la même clé ne crée pas d'action de plus, avant comme après le rejeu (inv. 18)
#[tokio::test]
async fn scenario_s7_same_keys_never_move_a_message_twice_via_cli() {
    let mut fx = start_move_world(CapabilitySet::full()).await;
    let first = fx.move_three_offline().await;
    let again = fx.move_three_offline().await;
    assert_eq!(first, again);
    assert_eq!(fx.queue_view().await.len(), 3);
    fx.reconnect_and_replay().await;
    let third = fx.move_three_offline().await;
    assert_eq!(first, third);
    fx.world.go_online();
    fx.world.run_sync_ok(&["sync", "start"]).await;
    let log = fx.world.bench.store.command_log();
    assert_eq!(
        log.iter()
            .filter(|line| line.starts_with("UID MOVE"))
            .count(),
        3
    );
    assert_eq!(fx.world.server_subjects("Archive").len(), 3);
}

/// Annuler un déplacement en attente le ramène en réception et le serveur ne le déplace jamais
#[tokio::test]
async fn scenario_s7_undo_of_an_offline_move_restores_the_message_via_cli() {
    let mut fx = start_move_world(CapabilitySet::full()).await;
    let actions = fx.move_three_offline().await;
    fx.world.run_ok(&["action", "undo", &actions[0]]).await;
    assert_eq!(fx.world.subjects_in(&fx.inbox).await, ["Delta", "Alpha"]);
    fx.reconnect_and_replay().await;
    assert_eq!(fx.world.server_subjects("Archive"), ["Gamma", "Beta"]);
    assert_eq!(fx.world.server_subjects("INBOX"), ["Alpha", "Delta"]);
}
