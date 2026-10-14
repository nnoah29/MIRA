//! Signaux d'arrêt du processus (SIGTERM, SIGINT, SIGHUP) transformés en demande de sortie propre.

use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinHandle;

/// Écoute les signaux d'arrêt et envoie une demande de sortie au premier reçu ; sans effet si l'écoute est impossible.
pub fn spawn_stop_signals(sender: UnboundedSender<()>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let (Ok(mut term), Ok(mut int), Ok(mut hup)) = (
            signal(SignalKind::terminate()),
            signal(SignalKind::interrupt()),
            signal(SignalKind::hangup()),
        ) else {
            return;
        };
        tokio::select! {
            _ = term.recv() => {}
            _ = int.recv() => {}
            _ = hup.recv() => {}
        }
        let _ = sender.send(());
    })
}
