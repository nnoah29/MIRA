#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `host_op shutdown` sur le vrai binaire : le moteur est fermé jusqu'au bout avant la sortie du processus.

mod support;

use std::time::Duration;

use mira_assembly::api::{EventRequest, Request};
use mira_assembly::mira_vocabulary::EventFilter;
use mira_host::Host;
use mira_protocol::{Frame, HostOp};
use support::process_args::{attach, start};

/// Après `shutdown`, le processus sort avec 0 et le journal du profil contient `engine_stopping`.
#[tokio::test(flavor = "multi_thread")]
async fn host_op_shutdown_closes_the_engine_before_the_process_exits() {
    let dir = tempfile::tempdir().unwrap();
    let mut daemon = start(dir.path(), &[]);
    let mut client = attach(dir.path()).await;
    client
        .send(Frame::HostOp {
            id: 1,
            op: HostOp::Shutdown,
        })
        .await;
    let status = daemon.wait_exit(Duration::from_secs(30));
    assert_eq!(status.code(), Some(0), "{:?}", daemon.output);
    let host = Host::new();
    let opened = host.open(support::config(&dir.path().join("home")));
    assert!(opened.contains("\"ok\":true"), "{opened}");
    let filter = EventFilter::default();
    let request = Request::Event(EventRequest::GetEvents { filter, limit: 500 });
    let events = host
        .execute(&serde_json::to_string(&request).unwrap())
        .await;
    assert!(
        events.contains("engine_stopping"),
        "journal sans engine_stopping : {events}"
    );
}
