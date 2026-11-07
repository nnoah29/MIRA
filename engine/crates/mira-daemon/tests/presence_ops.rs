#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Présence par la socket : `set_presence` répond ok, chaque connexion compte pour une, la fermeture retire la sienne.

mod support;

use std::time::Duration;

use mira_daemon::server::ServerContext;
use mira_protocol::Frame;
use support::{TestClient, TestDaemon};

/// Attend, au plus deux secondes, que la présence de l'hôte prenne cette valeur.
async fn wait_active(ctx: &ServerContext, wanted: bool) {
    for _ in 0..100 {
        if ctx.host.presence().is_active() == wanted {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("présence jamais {wanted}");
}

/// Envoie `set_presence` et exige un `result` ok de même identifiant.
async fn announce(client: &mut TestClient, id: u64, focused: bool) {
    client.send(Frame::SetPresence { id, focused }).await;
    let Some(Frame::Result { id: got, envelope }) = client.recv().await else {
        panic!("result attendu")
    };
    assert_eq!((got, &envelope["ok"]), (id, &true.into()));
}

/// Un client qui se déclare au premier plan rend l'hôte actif, puis inactif quand il n'y est plus.
#[tokio::test(flavor = "multi_thread")]
async fn set_presence_over_the_socket_marks_the_host_active() {
    let daemon = TestDaemon::start_closed().await;
    let mut client = TestClient::connect(&daemon).await;
    client.hello_ok(daemon.token()).await;
    announce(&mut client, 1, true).await;
    assert!(daemon.context().host.presence().is_active());
    announce(&mut client, 2, false).await;
    assert!(!daemon.context().host.presence().is_active());
}

/// La fermeture de la connexion retire la présence : le démon notifie de nouveau.
#[tokio::test(flavor = "multi_thread")]
async fn a_dropped_connection_removes_its_presence() {
    let daemon = TestDaemon::start_closed().await;
    let mut client = TestClient::connect(&daemon).await;
    client.hello_ok(daemon.token()).await;
    announce(&mut client, 1, true).await;
    drop(client);
    wait_active(&daemon.context(), false).await;
}

/// Deux connexions comptent séparément : le départ de l'une laisse l'autre active.
#[tokio::test(flavor = "multi_thread")]
async fn two_connections_are_counted_separately() {
    let daemon = TestDaemon::start_closed().await;
    let mut first = TestClient::connect(&daemon).await;
    first.hello_ok(daemon.token()).await;
    let mut second = TestClient::connect(&daemon).await;
    second.hello_ok(daemon.token()).await;
    announce(&mut first, 1, true).await;
    announce(&mut second, 1, true).await;
    drop(first);
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(daemon.context().host.presence().is_active());
    drop(second);
    wait_active(&daemon.context(), false).await;
}
