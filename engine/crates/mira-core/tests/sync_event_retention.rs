#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Chaque passe de synchronisation applique la rétention des événements (30 jours, 100 000 lignes).

mod support_sync;

use mira_core::sync::sync_account;
use mira_testkit::imap::CapabilitySet;
use mira_vocabulary::{EventKind, Timestamp};
use support_sync::provider::SyncTestProvider;
use support_sync::within;
use support_sync::world::{open_session, start_world};

/// Une passe efface les événements vieux de plus de 30 jours déjà présents dans la base du profil
#[tokio::test]
async fn regression_a_sync_pass_applies_the_event_retention() {
    within(async {
        let world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
        world
            .db
            .write(|tx| {
                tx.append_event(Timestamp::from_millis(1), &EventKind::EngineStopping)
                    .map(|_| ())
            })
            .unwrap();
        let mut session = open_session(&world).await;
        sync_account(&mut session).await.unwrap();
        let kept = world.db.read(|tx| tx.events_after(None, 100)).unwrap();
        assert!(kept.is_empty(), "{kept:?}");
    })
    .await;
}
