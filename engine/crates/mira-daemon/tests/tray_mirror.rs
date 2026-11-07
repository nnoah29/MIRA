#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! L'icône suit l'état annoncé par le serveur : démarrage, trousseau verrouillé, prêt.

mod support;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use mira_daemon::desktop::{TrayControl, TrayStatus, mirror_state};
use mira_protocol::HostState;
use support::unlock_harness::context;

/// Icône de test : garde les états qu'on lui demande d'afficher.
#[derive(Debug, Default)]
struct RecordingTray {
    seen: Mutex<Vec<TrayStatus>>,
}

impl TrayControl for RecordingTray {
    /// Note l'état.
    fn set_status(&self, status: TrayStatus) {
        self.seen.lock().unwrap().push(status);
    }
}

/// Les changements d'état de l'hôte arrivent à l'icône dans l'ordre, après l'état initial.
#[tokio::test(flavor = "multi_thread")]
async fn the_tray_follows_the_host_state() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = context(dir.path());
    let tray = Arc::new(RecordingTray::default());
    mirror_state(&ctx, tray.clone());
    for state in [HostState::KeyringLocked, HostState::Ready] {
        ctx.set_state(state);
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let seen = tray.seen.lock().unwrap().clone();
    let expected = [
        TrayStatus::Starting,
        TrayStatus::KeyringLocked,
        TrayStatus::UpToDate,
    ];
    assert_eq!(seen, expected);
}
