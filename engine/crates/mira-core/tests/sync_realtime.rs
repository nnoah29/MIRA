#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Temps réel (10.6, S5) : IDLE, relevé périodique en repli, coupure puis retour en ligne, arrêt propre.

mod support_sync;

use std::time::Duration;

use mira_testkit::imap::CapabilitySet;
use mira_vocabulary::AccountState;
use support_sync::events::announced;
use support_sync::inspect::{account_state, subjects};
use support_sync::mail::mail;
use support_sync::provider::SyncTestProvider;
use support_sync::supervised::start_supervised;
use support_sync::waiting::{
    back_online, diagnostics_for, finished_passes, wait_until, went_offline,
};
use support_sync::within;
use support_sync::world::start_world;

/// S5 : un message qui arrive pendant l'IDLE émet `NewMessageReceived` sans attendre le relevé périodique
#[tokio::test]
async fn scenario_s5_idle_new_message_emits_event() {
    within(async {
        let world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
        world
            .store
            .deliver("alice", "INBOX", &mail("Déjà là", "a@x"), &[]);
        let supervisor = start_supervised(&world, Duration::from_secs(600));
        wait_until(|| finished_passes(&world.recording) >= 1).await;
        wait_until(|| world.server.active_connections() == 2).await;
        world
            .store
            .deliver("alice", "INBOX", &mail("Tout frais", "b@x"), &[]);
        wait_until(|| announced(&world.recording) == 1).await;
        assert_eq!(subjects(&world.db), ["Déjà là", "Tout frais"]);
        assert!(supervisor.shutdown(Duration::from_secs(5)).await);
    })
    .await;
}

/// Sans IDLE, le relevé périodique trouve les nouveaux messages et le repli est journalisé
#[tokio::test]
async fn without_idle_polling_finds_new_messages_and_logs_the_fallback() {
    within(async {
        let world = start_world(CapabilitySet::minimal(), SyncTestProvider::folders()).await;
        let supervisor = start_supervised(&world, Duration::from_millis(100));
        wait_until(|| finished_passes(&world.recording) >= 1).await;
        world
            .store
            .deliver("alice", "INBOX", &mail("Par relevé", "p@x"), &[]);
        wait_until(|| announced(&world.recording) == 1).await;
        assert_eq!(diagnostics_for(&world.recording, "sync.realtime"), 1);
        assert_eq!(world.server.active_connections(), 1);
        assert!(supervisor.shutdown(Duration::from_secs(5)).await);
    })
    .await;
}

/// Une coupure réseau met le compte hors ligne, puis la reconnexion le remet en ligne et rattrape ce qui a été manqué
#[tokio::test]
async fn network_cut_goes_offline_then_back_online_and_catches_up() {
    within(async {
        let world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
        let supervisor = start_supervised(&world, Duration::from_secs(600));
        wait_until(|| finished_passes(&world.recording) >= 1).await;
        wait_until(|| world.server.active_connections() == 2).await;
        world.server.drop_all_connections();
        world
            .store
            .deliver("alice", "INBOX", &mail("Pendant la coupure", "c@x"), &[]);
        wait_until(|| back_online(&world.recording) == 1).await;
        wait_until(|| announced(&world.recording) == 1).await;
        assert_eq!(went_offline(&world.recording), 1);
        assert_eq!(account_state(&world.db), AccountState::Connected);
        assert!(supervisor.shutdown(Duration::from_secs(5)).await);
    })
    .await;
}

/// L'arrêt propre termine la tâche dans le délai et ferme toutes les connexions
#[tokio::test]
async fn shutdown_stops_the_task_and_closes_connections() {
    within(async {
        let world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
        let supervisor = start_supervised(&world, Duration::from_secs(600));
        wait_until(|| finished_passes(&world.recording) >= 1).await;
        assert_eq!(supervisor.active_tasks(), 1);
        assert!(supervisor.shutdown(Duration::from_secs(5)).await);
        assert_eq!(supervisor.active_tasks(), 0);
        wait_until(|| world.server.active_connections() == 0).await;
    })
    .await;
}
