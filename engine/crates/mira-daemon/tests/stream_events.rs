#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Flux d'événements : pas de rejeu sans `after`, fenêtre de crédits, `lagged` puis reprise, annulation.

mod support;

use std::time::Duration;

use mira_protocol::{EndReason, Frame};
use support::event_helpers::{attached, lagging};
use support::{Recv, TestClient};

const QUIET: Duration = Duration::from_millis(300);

/// Numéro du journal d'une trame `event`.
fn seq_of(frame: &Frame) -> u64 {
    let Frame::Event { event, .. } = frame else {
        panic!("event attendu : {frame:?}")
    };
    event["seq"].as_u64().unwrap()
}

/// Sans `after`, le journal passé n'est pas rejoué ; un nouvel événement arrive.
#[tokio::test(flavor = "multi_thread")]
async fn events_without_after_do_not_replay_the_journal() {
    let (_daemon, mut client, _home) = attached(|_| {}).await;
    client.make_event("avant").await;
    client.open_events(7, None, 8).await;
    assert!(matches!(client.recv_for(QUIET).await, Recv::Quiet));
    client.make_event("apres").await;
    let frame = client.recv().await.unwrap();
    assert!(matches!(frame, Frame::Event { id: 7, .. }), "{frame:?}");
}

/// Avec `after: 0`, le journal est rejoué ; la fenêtre de crédits est respectée à la trame près.
#[tokio::test(flavor = "multi_thread")]
async fn after_zero_replays_and_the_credit_window_is_respected() {
    let (_daemon, mut client, _home) = attached(|_| {}).await;
    for index in 0..5 {
        client.make_event(&format!("l{index}")).await;
    }
    client.open_events(7, Some(0), 1).await;
    assert!(matches!(client.recv().await, Some(Frame::Event { .. })));
    assert!(matches!(client.recv_for(QUIET).await, Recv::Quiet));
    client.send(Frame::Credit { id: 7, n: 2 }).await;
    assert_eq!(client.count_until_quiet(QUIET).await.len(), 2);
}

/// Un client qui ne lit plus reçoit `end lagged`, puis se réabonne avec `after` sans perte ni doublon.
#[tokio::test(flavor = "multi_thread")]
async fn a_client_that_stops_reading_gets_end_lagged_and_resubscribes_with_after() {
    let (daemon, mut client, _home) = attached(lagging).await;
    let mut producer = TestClient::connect(&daemon).await;
    producer.hello_ok(daemon.token()).await;
    client.open_events(7, None, 1).await;
    producer.make_event("premier").await;
    let first = seq_of(&client.recv().await.unwrap());
    for index in 0..8 {
        producer.make_event(&format!("suite{index}")).await;
    }
    let (_, reason) = client.collect_until_end(7).await;
    assert_eq!(reason, EndReason::Lagged);
    client.open_events(8, Some(first), 100).await;
    let frames = client.count_until_quiet(QUIET).await;
    let seqs: Vec<u64> = frames.iter().map(seq_of).collect();
    let expected: Vec<u64> = (first + 1..=first + seqs.len() as u64).collect();
    assert_eq!(seqs, expected, "ni trou ni doublon après la reprise");
    assert!(
        seqs.len() >= 8,
        "les huit événements manquants arrivent : {seqs:?}"
    );
}

/// `cancel` termine le flux d'événements par `end cancelled`.
#[tokio::test(flavor = "multi_thread")]
async fn cancel_ends_the_event_stream_with_cancelled() {
    let (_daemon, mut client, _home) = attached(|_| {}).await;
    client.open_events(7, None, 8).await;
    client.send(Frame::Cancel { id: 7 }).await;
    let (lines, reason) = client.collect_until_end(7).await;
    assert!(lines.is_empty());
    assert_eq!(reason, EndReason::Cancelled);
}
