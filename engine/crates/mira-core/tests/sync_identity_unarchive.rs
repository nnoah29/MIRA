#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Désarchivage fait ailleurs : la réception, synchronisée avant l'archive, ne recrée pas le message (D-13, inv. 7).

mod support_sync;

use mira_core::sync::sync_account;
use mira_testkit::imap::CapabilitySet;
use mira_vocabulary::{EventKind, MessageId};
use support_sync::mail::mail;
use support_sync::mirror::{ids_by_subject, mirror_rows, stored_messages};
use support_sync::provider::SyncTestProvider;
use support_sync::within;
use support_sync::world::{SyncWorld, open_session, start_world};

/// Monde sans identifiant stable dont l'archive contient « Contrat »
async fn world_with_archived_message() -> SyncWorld {
    let world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
    world
        .store
        .create_mailbox("alice", "Archive", Some("\\Archive"));
    world
        .store
        .deliver("alice", "Archive", &mail("Contrat", "k@x"), &[]);
    world
}

/// Remet le message d'« Archive » dans la réception depuis un autre appareil (commande MOVE)
async fn move_to_inbox(world: &SyncWorld, uid: u32) {
    let mut other = open_session(world).await;
    other.conn.client.select("Archive", None).await.unwrap();
    other
        .conn
        .client
        .move_messages(&[uid], "INBOX")
        .await
        .unwrap();
}

/// Identifiants des messages dont l'enregistreur a vu la suppression
fn deleted_ids(world: &SyncWorld) -> Vec<MessageId> {
    let deleted = |kind: EventKind| match kind {
        EventKind::MessageDeleted { message, .. } => Some(message),
        _ => None,
    };
    world
        .recording
        .kinds()
        .into_iter()
        .filter_map(deleted)
        .collect()
}

/// Désarchiver ailleurs garde l'identifiant interne : aucune suppression du message d'origine, aucun doublon
#[tokio::test]
async fn regression_unarchive_elsewhere_keeps_the_internal_id() {
    within(async {
        let world = world_with_archived_message().await;
        let mut session = open_session(&world).await;
        sync_account(&mut session).await.unwrap();
        let before = ids_by_subject(&world.db)["Contrat"].clone();
        move_to_inbox(&world, 1).await;
        sync_account(&mut session).await.unwrap();
        assert_eq!(mirror_rows(&world.db)[0].mailbox, "INBOX");
        assert_eq!(ids_by_subject(&world.db)["Contrat"], before);
        assert_eq!(stored_messages(&world.db), 1);
        assert!(!deleted_ids(&world).contains(&before));
    })
    .await;
}

/// Une vraie copie dans deux boîtes reste deux messages : sans orphelin, rien n'est fusionné
#[tokio::test]
async fn a_live_copy_in_two_mailboxes_is_never_merged() {
    within(async {
        let world = world_with_archived_message().await;
        world
            .store
            .deliver("alice", "INBOX", &mail("Contrat", "k@x"), &[]);
        let mut session = open_session(&world).await;
        sync_account(&mut session).await.unwrap();
        assert_eq!(stored_messages(&world.db), 2);
        sync_account(&mut session).await.unwrap();
        assert_eq!(stored_messages(&world.db), 2);
    })
    .await;
}

/// Le message conservé reçoit les drapeaux que le serveur lui donne dans sa nouvelle boîte
#[tokio::test]
async fn the_kept_message_takes_the_flags_of_its_new_place() {
    within(async {
        let world = world_with_archived_message().await;
        let mut session = open_session(&world).await;
        sync_account(&mut session).await.unwrap();
        move_to_inbox(&world, 1).await;
        world.store.set_flags("alice", "INBOX", 1, &["\\Seen"]);
        sync_account(&mut session).await.unwrap();
        assert!(mirror_rows(&world.db)[0].seen);
    })
    .await;
}
