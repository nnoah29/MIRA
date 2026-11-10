//! Demande d'arrêt d'un processus : SIGTERM sous Unix, rien ailleurs.

use tokio::process::Child;

/// Demande l'arrêt poliment (SIGTERM).
#[cfg(unix)]
pub(super) fn ask_to_stop(child: &Child) {
    if let Some(pid) = child
        .id()
        .and_then(|id| i32::try_from(id).ok())
        .and_then(rustix::process::Pid::from_raw)
    {
        let _ = rustix::process::kill_process(pid, rustix::process::Signal::TERM);
    }
}

/// Hors Unix, il n'y a pas de demande polie : l'arrêt forcé suit.
#[cfg(not(unix))]
pub(super) fn ask_to_stop(_child: &Child) {}
