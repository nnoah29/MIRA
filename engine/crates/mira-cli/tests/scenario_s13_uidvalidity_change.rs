#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! S13 par le CLI : le serveur renumérote une boîte ; recoupement, alerte, actions relues, doute = deux messages (P7, inv. 7, 23).

#[allow(dead_code)]
mod cli_support;

use cli_support::sb_events::events_of_type;
use cli_support::sb_world::SbWorld;
use mira_testkit::imap::CapabilitySet;

const USER: &str = "alice@folders.test";

/// Compte synchronisé sur trois courriers « Dossier 1 » à « Dossier 3 » ; rend le monde, le compte et la réception.
async fn numbered_world() -> (SbWorld, String, String) {
    let world = SbWorld::start(&[USER], CapabilitySet::full()).await;
    for number in 1..=3 {
        world.deliver_subject(USER, "INBOX", &format!("Dossier {number}"));
    }
    let account = world.add_account(USER).await;
    world.run_sync_ok(&["sync", "start"]).await;
    let inbox = world.mailbox_named(&account, "INBOX").await;
    (world, account, inbox)
}

/// Le premier courrier disparaît et le serveur renumérote la boîte (nouveau UIDVALIDITY).
fn renumber_after_expunge(world: &SbWorld) {
    world.store.expunge(USER, "INBOX", 1);
    world.store.renumber(USER, "INBOX");
}

/// Renumérotation vue à la synchro : mêmes identifiants, UID relus, repère mis à jour, alerte émise
#[tokio::test]
async fn scenario_s13_renumbering_keeps_internal_ids_and_alerts_via_cli() {
    let (world, account, inbox) = numbered_world().await;
    let second = world.message_id_of(&inbox, "Dossier 2").await;
    let third = world.message_id_of(&inbox, "Dossier 3").await;
    let validity = world.store.uidvalidity(USER, "INBOX").unwrap();
    renumber_after_expunge(&world);
    let renumbered = world.store.uidvalidity(USER, "INBOX").unwrap();
    assert_ne!(validity, renumbered);
    let synced = world.run_sync_ok(&["sync", "start"]).await;
    assert_eq!(synced["data"]["status"]["type"], "finished");
    assert_eq!(world.subjects_of(&inbox).await, ["Dossier 2", "Dossier 3"]);
    assert_eq!(world.message_id_of(&inbox, "Dossier 2").await, second);
    assert_eq!(world.message_id_of(&inbox, "Dossier 3").await, third);
    let boxes = world.run_ok(&["mailbox", "list", &account]).await;
    let listed = boxes["data"].as_array().unwrap();
    let found = listed.iter().find(|item| item["id"] == inbox.as_str());
    assert_eq!(found.unwrap()["marker"]["uid_validity"], renumbered);
    assert_eq!(world.counters_of(&account, "INBOX").await, (2, 2));
    let events = world.events().await;
    assert_eq!(events_of_type(&events, "mailbox_resynchronized").len(), 1);
}

/// Une action en attente visant la boîte est relue contre les nouvelles apparitions et touche le bon message
#[tokio::test]
async fn scenario_s13_pending_action_follows_the_new_numbers_via_cli() {
    let (world, account, inbox) = numbered_world().await;
    let third = world.message_id_of(&inbox, "Dossier 3").await;
    let second = world.message_id_of(&inbox, "Dossier 2").await;
    let action = world.apply_pending("mark-read", &third).await;
    assert_eq!(world.server_uid_of(USER, "INBOX", "Dossier 3"), 3);
    renumber_after_expunge(&world);
    assert_eq!(world.server_uid_of(USER, "INBOX", "Dossier 3"), 2);
    world.run_sync_ok(&["sync", "start"]).await;
    world.replay_queue_through(&second).await;
    assert!(world.server_has_flag(USER, "INBOX", "Dossier 3", "\\Seen"));
    assert!(!world.server_has_flag(USER, "INBOX", "Dossier 2", "\\Seen"));
    assert!(world.server_has_flag(USER, "INBOX", "Dossier 2", "\\Flagged"));
    let queued = world.queued_actions(&account).await;
    assert_eq!(queued[0]["id"], action.as_str());
    assert!(queued.iter().all(|queued| queued["state"] == "succeeded"));
}

/// Doute sur l'identité (deux courriers identiques) : deux messages gardés ; le courrier unique garde son identifiant
#[tokio::test]
async fn scenario_s13_ambiguous_match_keeps_two_messages_via_cli() {
    let (world, account, inbox) = numbered_world().await;
    let twin = SbWorld::raw_mail(USER, "Jumeau");
    world.store.deliver(USER, "INBOX", &twin, &[]);
    world.store.deliver(USER, "INBOX", &twin, &[]);
    world.run_sync_ok(&["sync", "start"]).await;
    assert_eq!(world.counters_of(&account, "INBOX").await, (5, 5));
    let unique = world.message_id_of(&inbox, "Dossier 1").await;
    world.store.renumber(USER, "INBOX");
    world.run_sync_ok(&["sync", "start"]).await;
    assert_eq!(world.counters_of(&account, "INBOX").await, (5, 5));
    let subjects = world.subjects_of(&inbox).await;
    assert_eq!(subjects.iter().filter(|s| *s == "Jumeau").count(), 2);
    assert_eq!(world.message_id_of(&inbox, "Dossier 1").await, unique);
}
