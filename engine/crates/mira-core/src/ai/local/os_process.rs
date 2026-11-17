//! Particularités du système pour le processus du serveur : arrêt poli sous Unix, console et environnement sous Windows.

use tokio::process::{Child, Command};

/// Demande l'arrêt poliment (SIGTERM) ; dit si la demande a pu partir.
#[cfg(unix)]
pub(super) fn ask_to_stop(child: &Child) -> bool {
    let Some(pid) = child
        .id()
        .and_then(|id| i32::try_from(id).ok())
        .and_then(rustix::process::Pid::from_raw)
    else {
        return false;
    };
    rustix::process::kill_process(pid, rustix::process::Signal::TERM).is_ok()
}

/// Hors Unix, il n'y a pas de demande polie : l'arrêt est direct.
#[cfg(not(unix))]
pub(super) fn ask_to_stop(_child: &Child) -> bool {
    false
}

/// Sans effet sous Unix.
#[cfg(unix)]
pub(super) fn tune_command(_command: &mut Command) {}

/// Sous Windows : pas de console, et `SystemRoot` conservé (les sockets et la CRT en ont besoin).
#[cfg(not(unix))]
pub(super) fn tune_command(command: &mut Command) {
    command.creation_flags(0x0800_0000);
    if let Some(root) = std::env::var_os("SystemRoot") {
        command.env("SystemRoot", root);
    }
}
