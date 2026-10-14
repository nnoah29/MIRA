#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! S11 par le CLI : un message présent sous deux libellés est un seul message à deux apparitions (D-13, D-14, inv. 7, 8, 12).

#[allow(dead_code)]
mod cli_support;

use cli_support::json_views::page_ids;
use cli_support::sb_world::SbWorld;
use mira_testkit::imap::CapabilitySet;

const USER: &str = "alice@gmail.com";

/// Compte synchronisé : « Facture » sous INBOX et Travail (mêmes identifiants serveur), « Seul » sous INBOX seulement.
async fn labelled_world() -> (SbWorld, String, String, String) {
    let capabilities = CapabilitySet::full().with_label_extensions();
    let world = SbWorld::start(&[USER], capabilities).await;
    world.store.create_mailbox(USER, "Travail", None);
    let invoice = SbWorld::raw_mail(USER, "Facture");
    let boxes = ["INBOX", "Travail"];
    world
        .store
        .deliver_with_labels(USER, &boxes, &invoice, 555, 900);
    world.deliver_subject(USER, "INBOX", "Seul");
    let account = world.add_account(USER).await;
    world.run_sync_ok(&["sync", "start"]).await;
    let inbox = world.mailbox_named(&account, "INBOX").await;
    let work = world.mailbox_named(&account, "Travail").await;
    (world, account, inbox, work)
}

/// Un seul message, deux apparitions : même identifiant sous les deux libellés, vue unifiée sans doublon
#[tokio::test]
async fn scenario_s11_message_in_two_labels_is_one_message_via_cli() {
    let (world, account, inbox, work) = labelled_world().await;
    assert_eq!(world.counters_of(&account, "INBOX").await, (2, 2));
    assert_eq!(world.counters_of(&account, "Travail").await, (1, 1));
    let id = world.message_id_of(&inbox, "Facture").await;
    assert_eq!(world.message_id_of(&work, "Facture").await, id);
    let detail = world.run_ok(&["message", "get", &id]).await;
    assert_eq!(
        detail["data"]["mailboxes"],
        serde_json::json!([inbox, work])
    );
    let unified = world
        .run_ok(&["message", "list", "--view", "unified"])
        .await;
    assert_eq!(page_ids(&unified).len(), 2);
    let mut subjects: Vec<&str> = unified["data"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["subject"].as_str().unwrap())
        .collect();
    subjects.sort_unstable();
    assert_eq!(subjects, ["Facture", "Seul"]);
    world.run_sync_ok(&["sync", "start"]).await;
    assert_eq!(world.message_id_of(&inbox, "Facture").await, id);
    assert_eq!(world.counters_of(&account, "INBOX").await, (2, 2));
}

/// Marquer lu agit sur le message, donc sur les deux apparitions, localement puis sur le serveur
#[tokio::test]
async fn scenario_s11_mark_read_applies_to_both_appearances_via_cli() {
    let (world, account, inbox, work) = labelled_world().await;
    let id = world.message_id_of(&inbox, "Facture").await;
    let pump = world.pump_clock();
    let line = ["action", "apply", "mark-read", "--message", &id, "--wait"];
    let applied = world.run(&line).await;
    pump.abort();
    assert_eq!(applied.code, 0, "{}", applied.stderr);
    for mailbox in [&inbox, &work] {
        let messages = world.messages_of(mailbox).await;
        let invoice = messages.iter().find(|m| m["id"] == id.as_str()).unwrap();
        assert_eq!(invoice["flags"]["seen"], true);
    }
    assert_eq!(world.counters_of(&account, "INBOX").await, (2, 1));
    assert_eq!(world.counters_of(&account, "Travail").await, (1, 0));
    assert!(world.server_seen(USER, "INBOX", 1));
    assert!(world.server_seen(USER, "Travail", 1));
    assert!(!world.server_seen(USER, "INBOX", 2));
    world.run_sync_ok(&["sync", "start"]).await;
    let detail = world.run_ok(&["message", "get", &id]).await;
    assert_eq!(detail["data"]["flags"]["seen"], true);
}

/// Retirer le message de « Travail » supprime cette apparition seulement ; le message reste dans INBOX
#[tokio::test]
async fn scenario_s11_removing_from_one_label_keeps_the_other_via_cli() {
    let (world, account, inbox, work) = labelled_world().await;
    let id = world.message_id_of(&inbox, "Facture").await;
    let pump = world.pump_clock();
    let line = [
        "action",
        "apply",
        "remove-from-mailbox",
        "--message",
        &id,
        "--to",
        &work,
        "--wait",
    ];
    let removed = world.run(&line).await;
    pump.abort();
    assert_eq!(removed.code, 0, "{}", removed.stderr);
    let detail = world.run_ok(&["message", "get", &id]).await;
    assert_eq!(detail["data"]["mailboxes"], serde_json::json!([inbox]));
    assert!(world.messages_of(&work).await.is_empty());
    assert_eq!(world.counters_of(&account, "INBOX").await, (2, 2));
    assert!(world.store.messages(USER, "Travail").is_empty());
    assert_eq!(world.server_uids(USER, "INBOX"), [1, 2]);
    world.run_sync_ok(&["sync", "start"]).await;
    assert_eq!(world.message_id_of(&inbox, "Facture").await, id);
    assert!(world.messages_of(&work).await.is_empty());
}
