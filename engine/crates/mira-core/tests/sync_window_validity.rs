#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! « Remonter plus loin » face à une boîte renumérotée : jamais d'UID de deux validités mêlés.

mod support_sync;

use mira_core::operations::OperationRegistry;
use mira_core::sync::{extend_window, sync_account};
use mira_testkit::imap::CapabilitySet;
use support_sync::inspect::window_days;
use support_sync::mail::{days_ago, mail};
use support_sync::mirror::{mailbox_named, mirror_rows};
use support_sync::provider::SyncTestProvider;
use support_sync::within;
use support_sync::world::{SyncWorld, open_session, start_world};

/// Monde synchronisé (plage de 90 jours) avec un message récent et un message vieux de 100 jours
async fn world_with_old_mail() -> SyncWorld {
    let world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
    let recent = mail("Récent", "recent@x");
    world
        .store
        .deliver_dated("alice", "INBOX", &recent, &[], days_ago(1));
    let old = mail("Ancien", "ancien@x");
    world
        .store
        .deliver_dated("alice", "INBOX", &old, &[], days_ago(100));
    let mut session = open_session(&world).await;
    sync_account(&mut session).await.unwrap();
    world
}

/// Si le serveur a renuméroté la boîte, l'extension est refusée et le miroir reste intact
#[tokio::test]
async fn regression_extend_refuses_a_changed_uid_validity() {
    within(async {
        let world = world_with_old_mail().await;
        world.store.renumber("alice", "INBOX");
        let inbox = mailbox_named(&world.db, "INBOX");
        let registry = OperationRegistry::new(world.ctx.ids.clone(), world.ctx.events.clone());
        let handle = registry.start("extend_window");
        let mut session = open_session(&world).await;
        let error = extend_window(&mut session, &handle, &inbox.id, 30)
            .await
            .unwrap_err();
        assert_eq!(error.code, "incompatible_state.mailbox_changed");
        assert_eq!(mirror_rows(&world.db).len(), 1);
        assert_eq!(window_days(&world.db), 90);
    })
    .await;
}

/// Sans renumérotation, l'extension va jusqu'au bout et élargit la plage
#[tokio::test]
async fn extend_succeeds_when_the_uid_validity_is_unchanged() {
    within(async {
        let world = world_with_old_mail().await;
        let inbox = mailbox_named(&world.db, "INBOX");
        let registry = OperationRegistry::new(world.ctx.ids.clone(), world.ctx.events.clone());
        let handle = registry.start("extend_window");
        let mut session = open_session(&world).await;
        extend_window(&mut session, &handle, &inbox.id, 30)
            .await
            .unwrap();
        assert_eq!(mirror_rows(&world.db).len(), 2);
        assert_eq!(window_days(&world.db), 120);
    })
    .await;
}
