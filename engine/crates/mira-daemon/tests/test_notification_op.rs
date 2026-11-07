#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Opération d'hôte `test_notification` : le démon affiche lui-même une notification d'essai.

mod support;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use mira_daemon::desktop::Desktop;
use mira_host::notifier::Texts;
use mira_protocol::{Frame, HostOp};
use support::notifier_sink::CollectingSink;
use support::{TestClient, TestDaemon};

/// Le démon appelle sa fonction d'essai et répond ok ; sans fonction posée, il répond ok sans rien faire.
#[tokio::test(flavor = "multi_thread")]
async fn the_op_runs_the_daemon_test_hook_and_answers_ok() {
    let daemon = TestDaemon::start_closed().await;
    let calls = Arc::new(AtomicUsize::new(0));
    let seen = calls.clone();
    daemon.context().test_notification.set(move || {
        seen.fetch_add(1, Ordering::SeqCst);
    });
    let mut client = TestClient::connect(&daemon).await;
    client.hello_ok(daemon.token()).await;
    client
        .send(Frame::HostOp {
            id: 4,
            op: HostOp::TestNotification,
        })
        .await;
    let Some(Frame::Result { id: 4, envelope }) = client.recv().await else {
        panic!("result attendu")
    };
    assert_eq!(envelope["ok"], true);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

/// La notification d'essai est muette de message, avec le texte de la langue.
#[test]
fn the_test_notification_uses_the_locale_texts() {
    let sink = Arc::new(CollectingSink::default());
    let desktop = Desktop::new(Some(sink.clone()), Texts::for_locale("fr"));
    desktop.announce_test();
    let shown = sink.shown();
    assert_eq!(shown[0].title, "MIRA");
    assert_eq!(shown[0].body, "Les notifications fonctionnent.");
    assert!(shown[0].message.is_none());
}
