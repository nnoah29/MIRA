#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Synchronisation incrémentale : le serveur gagne, suppressions ailleurs, nouveaux messages, rejeu local.

mod support_sync;

use mira_core::sync::{SyncSession, sync_account};
use mira_testkit::imap::CapabilitySet;
use support_sync::actions::queue_mark_read;
use support_sync::events::{announced, deleted};
use support_sync::inspect::{flags_of, subjects};
use support_sync::mail::mail;
use support_sync::mirror::{ids_by_subject, mailbox_named, mirror_rows};
use support_sync::provider::SyncTestProvider;
use support_sync::within;
use support_sync::world::{SyncWorld, deliver_many, open_session, start_world};

/// Monde avec trois messages dans la réception, déjà synchronisé une fois
async fn synced_world() -> (SyncWorld, SyncSession) {
    let world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
    deliver_many(&world, "INBOX", "Message", 3);
    let mut session = open_session(&world).await;
    sync_account(&mut session).await.unwrap();
    (world, session)
}

/// Invariant 21 : les drapeaux du serveur écrasent le miroir
#[tokio::test]
async fn invariant_21_server_wins_on_flags() {
    within(async {
        let (world, mut session) = synced_world().await;
        let target = ids_by_subject(&world.db)["Message 2"].clone();
        let mut local = flags_of(&world.db, &target);
        local.seen = true;
        world
            .db
            .write(|tx| tx.update_message_flags(&target, local))
            .unwrap();
        world.store.set_flags("alice", "INBOX", 2, &["\\Flagged"]);
        sync_account(&mut session).await.unwrap();
        let flags = flags_of(&world.db, &target);
        assert!(flags.flagged && !flags.seen, "le serveur gagne : {flags:?}");
    })
    .await;
}

/// D-23 : une action locale en attente est rejouée par-dessus les drapeaux du serveur
#[tokio::test]
async fn pending_local_action_is_replayed_over_server_flags() {
    within(async {
        let (world, mut session) = synced_world().await;
        let target = ids_by_subject(&world.db)["Message 1"].clone();
        queue_mark_read(&world, &target);
        world.store.set_flags("alice", "INBOX", 1, &["\\Flagged"]);
        sync_account(&mut session).await.unwrap();
        let flags = flags_of(&world.db, &target);
        assert!(flags.flagged, "le drapeau du serveur est pris");
        assert!(flags.seen, "l'action locale est rejouée par-dessus");
    })
    .await;
}

/// Un message supprimé depuis un autre appareil quitte le miroir, avec son événement
#[tokio::test]
async fn message_deleted_elsewhere_is_removed_from_the_mirror() {
    within(async {
        let (world, mut session) = synced_world().await;
        world.store.expunge("alice", "INBOX", 2);
        sync_account(&mut session).await.unwrap();
        assert_eq!(subjects(&world.db), ["Message 1", "Message 3"]);
        assert_eq!(deleted(&world.recording), 1);
        assert_eq!(mailbox_named(&world.db, "INBOX").total, 2);
    })
    .await;
}

/// Un nouveau message est annoncé une seule fois, même si la synchronisation est répétée
#[tokio::test]
async fn new_message_is_announced_once_and_never_duplicated() {
    within(async {
        let (world, mut session) = synced_world().await;
        world
            .store
            .deliver("alice", "INBOX", &mail("Message 4", "c4@x"), &[]);
        sync_account(&mut session).await.unwrap();
        sync_account(&mut session).await.unwrap();
        assert_eq!(mirror_rows(&world.db).len(), 4);
        assert_eq!(announced(&world.recording), 1);
    })
    .await;
}

/// Sans changement côté serveur, la synchronisation ne retélécharge aucun en-tête
#[tokio::test]
async fn unchanged_mailbox_downloads_no_header_again() {
    within(async {
        let (world, mut session) = synced_world().await;
        let before = world.store.command_log().len();
        sync_account(&mut session).await.unwrap();
        let log = world.store.command_log();
        assert!(
            log[before..].iter().all(|line| !line.contains("HEADER")),
            "{log:?}"
        );
        assert!(log[before..].iter().any(|line| line.contains("QRESYNC")));
    })
    .await;
}
