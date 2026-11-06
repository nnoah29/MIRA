//! Aides des tests d'événements : démon sur moteur vide avec un client attaché, réglage d'un client en retard.

use std::time::Duration;

use mira_daemon::server::ServerContext;

use super::{TestClient, TestDaemon, config};

/// Démon sur un moteur vide, avec un client attaché.
pub async fn attached(
    tune: impl FnOnce(&mut ServerContext),
) -> (TestDaemon, TestClient, tempfile::TempDir) {
    let home = tempfile::tempdir().unwrap();
    let daemon = TestDaemon::start_with(Some(config(home.path())), tune).await;
    let mut client = TestClient::connect(&daemon).await;
    client.hello_ok(daemon.token()).await;
    (daemon, client, home)
}

/// File de deux événements et délai de grâce de 100 ms : un client qui ne lit plus est vite en retard.
pub fn lagging(ctx: &mut ServerContext) {
    ctx.event_queue = 2;
    ctx.event_lag_grace = Duration::from_millis(100);
}
