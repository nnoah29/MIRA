#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Orphelins de passe : recoupement avant purge (D-13 cas 2, inv. 7) et purge différée si une boîte est ignorée.

mod support_sync;

use mira_core::sync::sync_account;
use mira_testkit::imap::CapabilitySet;
use mira_vocabulary::EventKind;
use support_sync::mail::mail;
use support_sync::mirror::{ids_by_subject, mirror_rows, stored_messages};
use support_sync::provider::SyncTestProvider;
use support_sync::scenarios::move_to_archive;
use support_sync::within;
use support_sync::world::{SyncWorld, open_session, start_world};

/// Monde sans identifiant stable (dossiers) dont la réception contient « Contrat », avec une boîte Archive
async fn world_with_one_message() -> SyncWorld {
    let world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
    world
        .store
        .create_mailbox("alice", "Archive", Some("\\Archive"));
    world
        .store
        .deliver("alice", "INBOX", &mail("Contrat", "k@x"), &[]);
    world
}

/// Nombre d'événements de suppression de message vus par l'enregistreur
fn deletions(world: &SyncWorld) -> usize {
    let kinds = world.recording.kinds();
    let deleted = |kind: &&EventKind| matches!(kind, EventKind::MessageDeleted { .. });
    kinds.iter().filter(deleted).count()
}

/// Un déplacement fait ailleurs sans identifiant stable garde l'identifiant interne (recoupement contre l'orphelin)
#[tokio::test]
async fn regression_move_elsewhere_without_stable_id_keeps_the_internal_id() {
    within(async {
        let world = world_with_one_message().await;
        let mut session = open_session(&world).await;
        sync_account(&mut session).await.unwrap();
        let before = ids_by_subject(&world.db)["Contrat"].clone();
        move_to_archive(&world, 1).await;
        sync_account(&mut session).await.unwrap();
        assert_eq!(mirror_rows(&world.db)[0].mailbox, "Archive");
        assert_eq!(ids_by_subject(&world.db)["Contrat"], before);
        assert_eq!(stored_messages(&world.db), 1);
        assert_eq!(deletions(&world), 0);
    })
    .await;
}

/// Une boîte ignorée sur erreur du serveur empêche la purge des orphelins de la passe
#[tokio::test]
async fn regression_skipped_mailbox_keeps_orphans_for_a_later_pass() {
    within(async {
        let world = world_with_one_message().await;
        let mut session = open_session(&world).await;
        sync_account(&mut session).await.unwrap();
        let before = ids_by_subject(&world.db)["Contrat"].clone();
        move_to_archive(&world, 1).await;
        world.store.fail_next("UID FETCH", "NO [SERVERBUG] panne");
        sync_account(&mut session).await.unwrap();
        assert_eq!(stored_messages(&world.db), 1);
        assert_eq!(deletions(&world), 0);
        sync_account(&mut session).await.unwrap();
        assert_eq!(ids_by_subject(&world.db)["Contrat"], before);
        assert_eq!(mirror_rows(&world.db)[0].mailbox, "Archive");
    })
    .await;
}

/// Un orphelin qui ne correspond à aucune arrivée est bien purgé à la fin d'une passe complète
#[tokio::test]
async fn orphan_without_a_match_is_purged_by_a_complete_pass() {
    within(async {
        let world = world_with_one_message().await;
        let mut session = open_session(&world).await;
        sync_account(&mut session).await.unwrap();
        world.store.expunge("alice", "INBOX", 1);
        sync_account(&mut session).await.unwrap();
        assert_eq!(stored_messages(&world.db), 0);
        assert_eq!(deletions(&world), 1);
    })
    .await;
}
