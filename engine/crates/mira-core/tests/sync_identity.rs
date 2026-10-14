#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Identité d'un message (D-13, P7) : deux apparitions, déplacement, renumérotation, doublons.

mod support_sync;

use mira_core::sync::sync_account;
use mira_testkit::imap::CapabilitySet;
use support_sync::events::resynchronized;
use support_sync::mail::mail;
use support_sync::mirror::{ids_by_subject, mirror_rows, stored_messages};
use support_sync::provider::SyncTestProvider;
use support_sync::scenarios::move_to_archive;
use support_sync::within;
use support_sync::world::{deliver_many, open_session, start_world};

/// S11 : un message présent sous deux libellés est un seul message avec deux apparitions
#[tokio::test]
async fn scenario_s11_message_in_two_labels_is_one_message() {
    within(async {
        let caps = CapabilitySet::full().with_label_extensions();
        let world = start_world(caps, SyncTestProvider::with_stable_ids()).await;
        let raw = mail("Facture", "f1@x");
        world
            .store
            .deliver_with_labels("alice", &["INBOX", "Travail"], &raw, 555, 900);
        let mut session = open_session(&world).await;
        sync_account(&mut session).await.unwrap();
        assert_eq!(stored_messages(&world.db), 1);
        let rows = mirror_rows(&world.db);
        let boxes: Vec<&str> = rows.iter().map(|r| r.mailbox.as_str()).collect();
        assert_eq!(boxes, ["INBOX", "Travail"]);
        let id = ids_by_subject(&world.db)["Facture"].clone();
        assert_eq!(
            world
                .db
                .read(|tx| tx.appearances_of_message(&id))
                .unwrap()
                .len(),
            2
        );
    })
    .await;
}

/// Invariant 7 : l'identifiant interne survit à un déplacement puis à une renumérotation
#[tokio::test]
async fn invariant_7_internal_id_stable_across_move_and_renumber() {
    within(async {
        let caps = CapabilitySet::full().with_label_extensions();
        let world = start_world(caps, SyncTestProvider::with_stable_ids()).await;
        world
            .store
            .create_mailbox("alice", "Archive", Some("\\Archive"));
        world
            .store
            .deliver_with_labels("alice", &["INBOX"], &mail("Contrat", "k@x"), 77, 8);
        let mut session = open_session(&world).await;
        sync_account(&mut session).await.unwrap();
        let before = ids_by_subject(&world.db)["Contrat"].clone();
        move_to_archive(&world, 1).await;
        sync_account(&mut session).await.unwrap();
        assert_eq!(mirror_rows(&world.db)[0].mailbox, "Archive");
        assert_eq!(ids_by_subject(&world.db)["Contrat"], before);
        world.store.renumber("alice", "Archive");
        sync_account(&mut session).await.unwrap();
        assert_eq!(ids_by_subject(&world.db)["Contrat"], before);
        assert_eq!(stored_messages(&world.db), 1);
    })
    .await;
}

/// S13 : un UIDVALIDITY différent resynchronise la boîte sans changer les identifiants des messages
#[tokio::test]
async fn scenario_s13_uidvalidity_change_resynchronizes_mailbox() {
    within(async {
        let world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
        deliver_many(&world, "INBOX", "Dossier", 3);
        let mut session = open_session(&world).await;
        sync_account(&mut session).await.unwrap();
        let before = ids_by_subject(&world.db);
        world.store.expunge("alice", "INBOX", 1);
        world.store.renumber("alice", "INBOX");
        sync_account(&mut session).await.unwrap();
        let after = ids_by_subject(&world.db);
        assert_eq!(after.len(), 2);
        assert_eq!(after["Dossier 2"], before["Dossier 2"]);
        assert_eq!(after["Dossier 3"], before["Dossier 3"]);
        let uids: Vec<u32> = mirror_rows(&world.db).iter().map(|r| r.uid).collect();
        assert_eq!(uids, [1, 2]);
        assert_eq!(resynchronized(&world.recording), 1);
        assert_eq!(stored_messages(&world.db), 2);
    })
    .await;
}

/// Invariant 23 : un recoupement ambigu garde deux messages plutôt que d'en fusionner à tort
#[tokio::test]
async fn invariant_23_ambiguous_fingerprint_keeps_two_messages() {
    within(async {
        let world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
        let twin = mail("Jumeau", "same@x");
        world.store.deliver("alice", "INBOX", &twin, &[]);
        world.store.deliver("alice", "INBOX", &twin, &[]);
        let mut session = open_session(&world).await;
        sync_account(&mut session).await.unwrap();
        assert_eq!(stored_messages(&world.db), 2);
        world.store.renumber("alice", "INBOX");
        sync_account(&mut session).await.unwrap();
        assert_eq!(stored_messages(&world.db), 2);
        assert_eq!(mirror_rows(&world.db).len(), 2);
    })
    .await;
}

/// Un message unique et sans ambiguïté est reconnu par recoupement après renumérotation
#[tokio::test]
async fn unambiguous_fingerprint_is_recognized_after_renumbering() {
    within(async {
        let world = start_world(CapabilitySet::minimal(), SyncTestProvider::folders()).await;
        world
            .store
            .deliver("alice", "INBOX", &mail("Unique", "u@x"), &[]);
        let mut session = open_session(&world).await;
        sync_account(&mut session).await.unwrap();
        let before = ids_by_subject(&world.db)["Unique"].clone();
        world.store.renumber("alice", "INBOX");
        sync_account(&mut session).await.unwrap();
        assert_eq!(ids_by_subject(&world.db)["Unique"], before);
    })
    .await;
}
