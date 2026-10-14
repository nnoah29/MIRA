//! Ce qui réveille la tâche d'un compte entre deux synchronisations.

use std::sync::Arc;
use std::time::Duration;

use super::runtime::{AccountRuntime, wait_shutdown};
use crate::sync::realtime::{IdleSignal, IdleWatcher};

/// Raison du réveil.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wake {
    Shutdown,
    Requested,
    InboxChanged,
    IdleLost,
    Poll,
}

/// Attend un signal de l'écoute IDLE ; sans écoute, n'arrive jamais.
async fn idle_signal(idle: &mut Option<IdleWatcher>) -> Wake {
    let Some(watcher) = idle.as_mut() else {
        return std::future::pending().await;
    };
    match watcher.signals.recv().await {
        Some(IdleSignal::Changed) => Wake::InboxChanged,
        _ => Wake::IdleLost,
    }
}

/// Attend l'arrêt, une demande, un changement signalé par IDLE ou l'échéance du relevé.
pub async fn wait_for_wake(
    runtime: &mut AccountRuntime,
    idle: &mut Option<IdleWatcher>,
    poll: Duration,
) -> Wake {
    let requests = Arc::clone(&runtime.requests);
    tokio::select! {
        () = wait_shutdown(&mut runtime.shutdown) => Wake::Shutdown,
        () = requests.notified() => Wake::Requested,
        wake = idle_signal(idle) => wake,
        () = tokio::time::sleep(poll) => Wake::Poll,
    }
}
