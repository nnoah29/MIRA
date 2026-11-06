#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Opérations d'hôte : `shutdown` arrête proprement, l'état annoncé suit l'hôte.

mod support;

use std::time::Duration;

use mira_host::Host;
use mira_protocol::{Frame, HostOp, HostState};
use support::{Recv, TestClient, TestDaemon, config};

/// `shutdown` répond ok, puis `serve` rend la main, le moteur est fermé et le profil libéré.
#[tokio::test(flavor = "multi_thread")]
async fn shutdown_answers_ok_then_serve_returns_and_the_engine_is_closed() {
    let home = tempfile::tempdir().unwrap();
    let mut daemon = TestDaemon::start(config(home.path())).await;
    let mut client = TestClient::connect(&daemon).await;
    client.hello_ok(daemon.token()).await;
    client
        .send(Frame::HostOp {
            id: 1,
            op: HostOp::Shutdown,
        })
        .await;
    let Some(Frame::Result { id: 1, envelope }) = client.recv().await else {
        panic!("result attendu")
    };
    assert_eq!(envelope["ok"], true);
    tokio::time::timeout(Duration::from_secs(10), daemon.task())
        .await
        .unwrap()
        .unwrap();
    assert!(daemon.context().host.engine().is_none());
    let reopened = Host::new().open(config(home.path()));
    assert!(reopened.contains("\"ok\":true"), "{reopened}");
}

/// Les autres connexions sont fermées, la socket disparaît et plus personne ne peut se connecter.
#[tokio::test(flavor = "multi_thread")]
async fn shutdown_closes_the_other_connections_and_refuses_new_ones() {
    let home = tempfile::tempdir().unwrap();
    let mut daemon = TestDaemon::start(config(home.path())).await;
    let mut other = TestClient::connect(&daemon).await;
    other.hello_ok(daemon.token()).await;
    let mut client = TestClient::connect(&daemon).await;
    client.hello_ok(daemon.token()).await;
    client
        .send(Frame::HostOp {
            id: 1,
            op: HostOp::Shutdown,
        })
        .await;
    assert!(matches!(
        other.recv_for(Duration::from_secs(5)).await,
        Recv::Closed
    ));
    tokio::time::timeout(Duration::from_secs(10), daemon.task())
        .await
        .unwrap()
        .unwrap();
    assert!(
        tokio::net::UnixStream::connect(daemon.socket_path())
            .await
            .is_err()
    );
}

/// `hello_ok` annonce l'état courant de l'hôte.
#[tokio::test(flavor = "multi_thread")]
async fn hello_ok_reports_starting_then_ready() {
    let daemon = TestDaemon::start_closed().await;
    let mut first = TestClient::connect(&daemon).await;
    let Frame::HelloOk { host_state, .. } = first.hello_ok(daemon.token()).await else {
        panic!()
    };
    assert_eq!(host_state, HostState::Starting);
    daemon.context().set_state(HostState::Ready);
    let mut second = TestClient::connect(&daemon).await;
    let Frame::HelloOk { host_state, .. } = second.hello_ok(daemon.token()).await else {
        panic!()
    };
    assert_eq!(host_state, HostState::Ready);
}

/// Si le moteur reste tenu pendant l'arrêt, `shutdown::run` rend l'enveloppe d'échec de la fermeture.
#[tokio::test(flavor = "multi_thread")]
async fn shutdown_reports_a_close_refused_because_the_engine_is_still_held() {
    let home = tempfile::tempdir().unwrap();
    let daemon = TestDaemon::start_with(Some(config(home.path())), |ctx| {
        ctx.shutdown_wait = Duration::from_millis(200);
    })
    .await;
    let held = daemon.context().host.engine();
    let outcome = mira_daemon::shutdown::run(daemon.context()).await;
    assert!(outcome.contains("moteur encore utilisé"), "{outcome}");
    drop(held);
}

/// Deux arrêts simultanés rendent le même résultat, une fois la fermeture terminée et le profil libéré.
#[tokio::test(flavor = "multi_thread")]
async fn two_concurrent_shutdowns_both_wait_for_the_close() {
    let home = tempfile::tempdir().unwrap();
    let daemon = TestDaemon::start(config(home.path())).await;
    let (first, second) = tokio::join!(
        mira_daemon::shutdown::run(daemon.context()),
        mira_daemon::shutdown::run(daemon.context())
    );
    assert_eq!(first, second);
    assert!(first.contains("\"ok\":true"), "{first}");
    let reopened = Host::new().open(config(home.path()));
    assert!(reopened.contains("\"ok\":true"), "{reopened}");
}
