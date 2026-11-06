#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Flux de chat : lignes dans l'ordre, suspension sans crédit, annulation par `cancel` et par fermeture.

mod support;

use std::time::Duration;

use mira_protocol::{EndReason, Frame};
use mira_testkit::ai::openai_stream;
use support::ai_daemon::AiDaemon;
use support::chat_fixture::chat_json;
use support::chat_helpers::{operation_status, slow_stream, wait_provider_closed};
use support::{Recv, TestDaemon};

const QUIET: Duration = Duration::from_millis(300);

/// Les lignes suivent `started`, `text`, `text`, `usage`, `done`, puis `end finished`.
#[tokio::test(flavor = "multi_thread")]
async fn chat_lines_arrive_as_started_text_usage_done_then_end_finished() {
    let ai = AiDaemon::start().await;
    ai.server
        .queue(openai_stream(&["Bon", "jour"], Some((12, 5))));
    let mut client = ai.attach().await;
    client
        .open_chat(3, &chat_json(Some(&ai.connection), "Salut"), 16)
        .await;
    let (lines, reason) = client.collect_until_end(3).await;
    let kinds: Vec<&str> = lines.iter().map(|l| l["type"].as_str().unwrap()).collect();
    assert_eq!(
        kinds,
        ["started", "text", "text", "usage", "done"],
        "{lines:?}"
    );
    assert_eq!(lines[3]["input_tokens"], 12);
    assert_eq!(reason, EndReason::Finished);
}

/// Sans crédit, la génération est suspendue, pas abandonnée : le reste arrive quand on en donne.
#[tokio::test(flavor = "multi_thread")]
async fn a_chat_stream_without_credit_is_suspended_not_dropped() {
    let ai = AiDaemon::start().await;
    ai.server
        .queue(openai_stream(&["Bon", "jour"], Some((12, 5))));
    let mut client = ai.attach().await;
    client
        .open_chat(3, &chat_json(Some(&ai.connection), "Salut"), 1)
        .await;
    let started = client.recv().await.unwrap();
    assert!(matches!(started, Frame::ChatDelta { id: 3, ref line } if line["type"] == "started"));
    assert!(matches!(client.recv_for(QUIET).await, Recv::Quiet));
    assert_eq!(ai.server.connections_closed_by_client(), 0);
    client.send(Frame::Credit { id: 3, n: 8 }).await;
    let (lines, reason) = client.collect_until_end(3).await;
    assert_eq!(lines.last().unwrap()["type"], "done", "{lines:?}");
    assert_eq!(reason, EndReason::Finished);
}

/// `cancel` en cours de route annule l'opération, ferme la connexion au fournisseur et finit par `end cancelled`.
#[tokio::test(flavor = "multi_thread")]
async fn cancel_mid_stream_cancels_the_operation() {
    let ai = AiDaemon::start().await;
    ai.server.queue(slow_stream());
    let mut client = ai.attach().await;
    client
        .open_chat(3, &chat_json(Some(&ai.connection), "Salut"), 64)
        .await;
    let Some(Frame::ChatDelta { line, .. }) = client.recv().await else {
        panic!("started attendu")
    };
    let operation = line["operation"].as_str().unwrap().to_string();
    client.recv().await.unwrap();
    client.send(Frame::Cancel { id: 3 }).await;
    let (_, reason) = client.collect_until_end(3).await;
    assert_eq!(reason, EndReason::Cancelled);
    wait_provider_closed(&ai).await;
    assert_eq!(
        operation_status(&mut client, &operation).await["type"],
        "cancelled"
    );
}

/// Fermer la connexion en cours de route a le même effet, vérifié par un second client.
#[tokio::test(flavor = "multi_thread")]
async fn closing_the_connection_mid_stream_cancels_the_operation() {
    let ai = AiDaemon::start().await;
    ai.server.queue(slow_stream());
    let mut client = ai.attach().await;
    client
        .open_chat(3, &chat_json(Some(&ai.connection), "Salut"), 64)
        .await;
    let Some(Frame::ChatDelta { line, .. }) = client.recv().await else {
        panic!("started attendu")
    };
    let operation = line["operation"].as_str().unwrap().to_string();
    drop(client);
    wait_provider_closed(&ai).await;
    let mut other = ai.attach().await;
    assert_eq!(
        operation_status(&mut other, &operation).await["type"],
        "cancelled"
    );
}

/// Sur un hôte jamais ouvert, une seule ligne d'erreur fixe puis `end finished`.
#[tokio::test(flavor = "multi_thread")]
async fn a_chat_on_a_closed_host_gets_one_error_line_then_end() {
    let daemon = TestDaemon::start_closed().await;
    let mut client = support::TestClient::connect(&daemon).await;
    client.hello_ok(daemon.token()).await;
    client.open_chat(3, &chat_json(None, "Salut"), 8).await;
    let (lines, reason) = client.collect_until_end(3).await;
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0]["error"]["message"], "moteur non ouvert");
    assert_eq!(reason, EndReason::Finished);
}
