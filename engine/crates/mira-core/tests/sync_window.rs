#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Plage locale (D-21, S12) : « remonter plus loin » en tâche longue, avec progression et annulation.

mod support_sync;

use mira_core::operations::{OperationRegistry, OperationStatus};
use mira_core::sync::{
    ConnectionBudget, ExtendOutcome, extend_window, start_extend_window, sync_account,
};
use mira_testkit::imap::CapabilitySet;
use mira_vocabulary::EventKind;
use support_sync::inspect::{subjects, window_days};
use support_sync::mail::{days_ago, mail};
use support_sync::mirror::{mailbox_named, mirror_rows};
use support_sync::provider::SyncTestProvider;
use support_sync::waiting::{count_events, wait_until};
use support_sync::within;
use support_sync::world::{SyncWorld, open_session, start_world};

/// Monde avec deux messages récents, trois de 100 jours et un de 200 jours, déjà synchronisé (plage de 90 jours)
async fn aged_world() -> (SyncWorld, OperationRegistry) {
    let world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
    let ages = [
        (1, "Récent A"),
        (2, "Récent B"),
        (100, "Ancien A"),
        (100, "Ancien B"),
        (101, "Ancien C"),
        (200, "Très ancien"),
    ];
    for (index, (age, subject)) in ages.into_iter().enumerate() {
        let raw = mail(subject, &format!("age{index}@x"));
        world
            .store
            .deliver_dated("alice", "INBOX", &raw, &[], days_ago(age));
    }
    let mut session = open_session(&world).await;
    sync_account(&mut session).await.unwrap();
    let registry = OperationRegistry::new(world.ctx.ids.clone(), world.ctx.events.clone());
    (world, registry)
}

/// S12 : remonter de 30 jours télécharge les en-têtes plus anciens, avec progression, et élargit la plage
#[tokio::test]
async fn scenario_s12_extend_window_downloads_older_headers() {
    within(async {
        let (world, registry) = aged_world().await;
        assert_eq!(mirror_rows(&world.db).len(), 2);
        let inbox = mailbox_named(&world.db, "INBOX");
        let (ctx, credentials) = (world.ctx.clone(), world.credentials.clone());
        let budget = ConnectionBudget::new(3);
        let operation = start_extend_window(ctx, budget, credentials, &registry, inbox.id, 30);
        wait_until(|| registry.get(&operation).unwrap().status == OperationStatus::Finished).await;
        let found = subjects(&world.db);
        assert_eq!(found.len(), 5, "{found:?}");
        assert!(!found.contains(&"Très ancien".to_string()));
        assert_eq!(window_days(&world.db), 120);
        let done = |k: &EventKind| {
            matches!(
                k,
                EventKind::OperationProgress {
                    done: 3,
                    total: 3,
                    ..
                }
            )
        };
        assert_eq!(count_events(&world.recording, done), 1);
    })
    .await;
}

/// Une extension annulée ne télécharge rien de plus et laisse la plage du compte inchangée
#[tokio::test]
async fn cancelled_extension_keeps_the_window_unchanged() {
    within(async {
        let (world, registry) = aged_world().await;
        let inbox = mailbox_named(&world.db, "INBOX");
        let mut session = open_session(&world).await;
        let handle = registry.start("extend_window");
        registry.cancel(handle.id()).unwrap();
        let outcome = extend_window(&mut session, &handle, &inbox.id, 30)
            .await
            .unwrap();
        handle.complete(Ok(()));
        assert_eq!(outcome, ExtendOutcome::Cancelled);
        assert_eq!(
            registry.get(handle.id()).unwrap().status,
            OperationStatus::Cancelled
        );
        assert_eq!(mirror_rows(&world.db).len(), 2);
        assert_eq!(window_days(&world.db), 90);
    })
    .await;
}
