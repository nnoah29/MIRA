//! Superviseur de test : délais courts, relevé périodique réglable.

use std::time::Duration;

use mira_core::sync::{BackoffPolicy, SupervisorConfig, SyncSupervisor};

use super::world::SyncWorld;

/// Délais très courts : reconnexion entre 50 et 200 ms, relevé au délai donné
pub fn quick_config(poll: Duration) -> SupervisorConfig {
    SupervisorConfig {
        backoff: BackoffPolicy {
            initial: Duration::from_millis(50),
            max: Duration::from_millis(200),
        },
        idle_renewal: Duration::from_secs(30),
        poll_interval: Some(poll),
    }
}

/// Superviseur dont la tâche du compte de test est déjà lancée
pub fn start_supervised(world: &SyncWorld, poll: Duration) -> SyncSupervisor {
    let supervisor = SyncSupervisor::new(quick_config(poll));
    let started = supervisor.start_account(world.ctx.clone(), world.credentials.clone());
    assert!(started, "la tâche du compte devait démarrer");
    supervisor
}
