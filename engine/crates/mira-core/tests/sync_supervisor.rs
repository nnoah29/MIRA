#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Supervision : une tâche par compte (invariant 24), connexions dans la limite du provider (invariant 25), authentification.

mod support_sync;

use std::sync::Arc;
use std::time::Duration;

use mira_testkit::imap::CapabilitySet;
use mira_vocabulary::{AccountState, EventKind};
use support_sync::credentials::TestCredentials;
use support_sync::inspect::{account_state, subjects};
use support_sync::mail::mail;
use support_sync::provider::SyncTestProvider;
use support_sync::scenarios::busy_account;
use support_sync::supervised::start_supervised;
use support_sync::waiting::{diagnostics_for, finished_passes, wait_until};
use support_sync::within;
use support_sync::world::start_world;

/// Plus grand nombre de passes ouvertes en même temps, d'après la suite d'événements
fn max_overlap(kinds: &[EventKind]) -> i32 {
    let (mut open, mut peak) = (0, 0);
    for kind in kinds {
        match kind {
            EventKind::SyncStarted { .. } => open += 1,
            EventKind::SyncFinished { .. } | EventKind::SyncFailed { .. } => open -= 1,
            _ => {}
        }
        peak = peak.max(open);
    }
    peak
}

/// Invariant 24 : une seule tâche par compte ; les demandes pendant une synchro ne lancent jamais de seconde passe en parallèle
#[tokio::test]
async fn invariant_24_single_sync_task_per_account() {
    within(async {
        let world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
        world
            .store
            .deliver("alice", "INBOX", &mail("Un", "1@x"), &[]);
        let supervisor = start_supervised(&world, Duration::from_secs(600));
        let again = supervisor.start_account(world.ctx.clone(), world.credentials.clone());
        assert!(!again, "un second démarrage ne crée pas de seconde tâche");
        for _ in 0..25 {
            assert!(supervisor.request_sync(&world.ctx.account.id));
        }
        wait_until(|| finished_passes(&world.recording) >= 2).await;
        tokio::time::sleep(Duration::from_millis(300)).await;
        assert_eq!(supervisor.active_tasks(), 1);
        assert_eq!(max_overlap(&world.recording.kinds()), 1);
        assert!(
            finished_passes(&world.recording) < 25,
            "demandes fusionnées"
        );
        assert!(supervisor.shutdown(Duration::from_secs(5)).await);
    })
    .await;
}

/// Invariant 25 : jamais plus de connexions que la limite du provider, tâche longue comprise (limite de 2 puis de 3)
#[tokio::test]
async fn invariant_25_connections_within_provider_limit() {
    within(async {
        for limit in [2_u32, 3] {
            let run = busy_account(limit).await;
            let allowed = usize::try_from(limit).unwrap();
            assert!(run.peak <= allowed, "limite {limit} : pic {}", run.peak);
            assert!(run.total <= allowed, "limite {limit} : total {}", run.total);
            assert_eq!(subjects(&run.world.db).len(), 2, "l'extension a abouti");
        }
    })
    .await;
}

/// Avec une seule connexion permise, ou sans permis de réserve, l'IDLE cède la place au relevé périodique
#[tokio::test]
async fn too_small_a_budget_falls_back_to_polling() {
    within(async {
        for limit in [1_u32, 2] {
            let provider = SyncTestProvider::folders().max_connections(limit);
            let world = start_world(CapabilitySet::full(), provider).await;
            let supervisor = start_supervised(&world, Duration::from_millis(100));
            wait_until(|| finished_passes(&world.recording) >= 1).await;
            world
                .store
                .deliver("alice", "INBOX", &mail("Relevé", "r@x"), &[]);
            wait_until(|| subjects(&world.db).len() == 1).await;
            assert_eq!(world.server.active_connections(), 1);
            assert_eq!(diagnostics_for(&world.recording, "sync.realtime"), 1);
            assert!(supervisor.shutdown(Duration::from_secs(5)).await);
        }
    })
    .await;
}

/// Des identifiants refusés mettent le compte en « authentification requise » sans le faire boucler
#[tokio::test]
async fn rejected_credentials_set_auth_required_and_stop_retrying() {
    within(async {
        let mut world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
        world.credentials = Arc::new(TestCredentials::rejected());
        let supervisor = start_supervised(&world, Duration::from_secs(600));
        wait_until(|| account_state(&world.db) == AccountState::AuthRequired).await;
        tokio::time::sleep(Duration::from_millis(300)).await;
        assert_eq!(
            world.server.connection_count(),
            1,
            "pas de nouvelle tentative"
        );
        assert_eq!(finished_passes(&world.recording), 0);
        assert!(supervisor.shutdown(Duration::from_secs(5)).await);
    })
    .await;
}

/// Un refus d'authentification suivi d'un rafraîchissement réussi n'interrompt pas la synchronisation
#[tokio::test]
async fn refresh_repairs_credentials_without_leaving_the_connected_state() {
    within(async {
        let mut world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
        world.credentials = Arc::new(TestCredentials::repaired_by_refresh());
        world
            .store
            .deliver("alice", "INBOX", &mail("Avec jeton", "j@x"), &[]);
        let supervisor = start_supervised(&world, Duration::from_secs(600));
        wait_until(|| finished_passes(&world.recording) >= 1).await;
        assert_eq!(subjects(&world.db), ["Avec jeton"]);
        assert_eq!(account_state(&world.db), AccountState::Connected);
        assert!(supervisor.shutdown(Duration::from_secs(5)).await);
    })
    .await;
}
