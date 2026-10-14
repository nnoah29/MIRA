#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! S8 par le CLI : message supprimé ailleurs, action en attente échouée, effet local annulé (D-23, inv. 21, 22).

#[allow(dead_code)]
mod cli_support;

use cli_support::sb_events::{action_by_id, events_of_type, first_seq_of_type};
use cli_support::sb_leaks::assert_outputs_clean;
use cli_support::sb_world::{SB_PASSWORD, SbWorld};
use mira_testkit::imap::CapabilitySet;

const USER: &str = "alice@folders.test";

/// Compte synchronisé avec Alpha (UID 1) et Beta (UID 2) en réception ; rend le monde, le compte et la réception.
async fn synced_world() -> (SbWorld, String, String) {
    let world = SbWorld::start(&[USER], CapabilitySet::full()).await;
    world.deliver_subject(USER, "INBOX", "Alpha");
    world.deliver_subject(USER, "INBOX", "Beta");
    let account = world.add_account(USER).await;
    world.run_sync_ok(&["sync", "start"]).await;
    let inbox = world.mailbox_named(&account, "INBOX").await;
    (world, account, inbox)
}

/// Marquer lu hors ligne, message supprimé ailleurs : la synchro le retire, l'action échoue, l'utilisateur est informé
#[tokio::test]
async fn scenario_s8_mark_read_on_message_deleted_elsewhere_fails_via_cli() {
    let (world, account, inbox) = synced_world().await;
    let alpha = world.message_id_of(&inbox, "Alpha").await;
    let beta = world.message_id_of(&inbox, "Beta").await;
    let action = world.apply_pending("mark-read", &alpha).await;
    let seen = world.run_ok(&["message", "get", &alpha]).await;
    assert_eq!(seen["data"]["flags"]["seen"], true);
    world.store.expunge(USER, "INBOX", 1);
    let synced = world.run_sync_ok(&["sync", "start"]).await;
    assert_eq!(synced["data"]["status"]["type"], "finished");
    assert_eq!(world.subjects_of(&inbox).await, ["Beta"]);
    let gone = world.run(&["message", "get", &alpha]).await;
    assert_eq!(gone.code, 11, "{}", gone.stderr);
    let pending = action_by_id(&world.queued_actions(&account).await, &action);
    assert_eq!(pending["state"], "pending");
    world.replay_queue_through(&beta).await;
    let failed = action_by_id(&world.queued_actions(&account).await, &action);
    assert_eq!(failed["state"], "failed");
    let reason = failed["last_error"].as_str().unwrap();
    assert!(reason.starts_with("not_found.message"), "{reason}");
}

/// Après l'échec : événement explicite, file qui continue, rien d'envoyé pour le message disparu
#[tokio::test]
async fn scenario_s8_failure_is_announced_and_the_queue_goes_on_via_cli() {
    let (world, account, inbox) = synced_world().await;
    let alpha = world.message_id_of(&inbox, "Alpha").await;
    let beta = world.message_id_of(&inbox, "Beta").await;
    let action = world.apply_pending("mark-read", &alpha).await;
    world.store.expunge(USER, "INBOX", 1);
    world.run_sync_ok(&["sync", "start"]).await;
    world.replay_queue_through(&beta).await;
    let events = world.events().await;
    let failures = events_of_type(&events, "action_failed");
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0]["action"], action.as_str());
    assert_eq!(failures[0]["code"], "not_found.message");
    assert!(events_of_type(&events, "message_deleted")[0]["message"] == alpha.as_str());
    let failed_at = first_seq_of_type(&events, "action_failed").unwrap();
    let applied_at = first_seq_of_type(&events, "action_applied").unwrap();
    assert!(failed_at < applied_at);
    let actions = world.queued_actions(&account).await;
    assert_eq!(actions.len(), 2);
    assert_eq!(actions[1]["state"], "succeeded");
    assert_eq!(world.server_uids(USER, "INBOX"), [2]);
    assert!(!world.server_seen(USER, "INBOX", 2));
}

/// Si la suppression n'est pas encore connue, le rejeu échoue et le déplacement local est annulé (inv. 22)
#[tokio::test]
async fn scenario_s8_archive_reverts_the_local_move_when_the_message_is_gone_via_cli() {
    let (world, account, inbox) = synced_world().await;
    let archive = world.mailbox_named(&account, "Archive").await;
    let alpha = world.message_id_of(&inbox, "Alpha").await;
    let beta = world.message_id_of(&inbox, "Beta").await;
    let action = world.apply_pending("archive", &alpha).await;
    let moved = world.run_ok(&["message", "get", &alpha]).await;
    assert_eq!(moved["data"]["mailboxes"], serde_json::json!([archive]));
    world.store.expunge(USER, "INBOX", 1);
    world.replay_queue_through(&beta).await;
    let failed = action_by_id(&world.queued_actions(&account).await, &action);
    assert_eq!(failed["state"], "failed");
    let reverted = world.run_ok(&["message", "get", &alpha]).await;
    assert_eq!(reverted["data"]["mailboxes"], serde_json::json!([inbox]));
    world.run_sync_ok(&["sync", "start"]).await;
    assert_eq!(world.subjects_of(&inbox).await, ["Beta"]);
    assert!(world.subjects_of(&archive).await.is_empty());
}

/// Aucune sortie, aucun fichier du profil ne contient le mot de passe pendant tout le scénario
#[tokio::test]
async fn scenario_s8_no_secret_leaks_in_outputs_or_profile_via_cli() {
    let (world, account, inbox) = synced_world().await;
    let alpha = world.message_id_of(&inbox, "Alpha").await;
    let beta = world.message_id_of(&inbox, "Beta").await;
    world.apply_pending("mark-read", &alpha).await;
    world.store.expunge(USER, "INBOX", 1);
    world.run_sync_ok(&["sync", "start"]).await;
    world.replay_queue_through(&beta).await;
    let table = world.run(&["queue", "list", &account]).await;
    let json = world.run(&["events", "--json"]).await;
    let status = world.run(&["sync", "status", "--json"]).await;
    assert_outputs_clean(&[&table, &json, &status], SB_PASSWORD);
    assert!(table.stdout.contains("failed"), "{}", table.stdout);
    assert!(!world.profile_contains(SB_PASSWORD));
    assert_eq!(world.secret_count(), 1);
}
