#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Pont `chat_stream` de bout en bout : lignes `started`, deltas, annulation à la fermeture du flux.

mod support;

use std::time::Duration;

use mira_bridge::runtime;
use mira_testkit::ai::{Behavior, openai_stream};
use serde_json::Value;
use support::chat_fixture::{chat_json, collect_lines, operation_status, start};

/// Flux OpenAI de quarante morceaux espacés de 100 ms, assez lent pour être annulé en route.
fn slow_stream() -> Behavior {
    let texts: Vec<String> = (0..40).map(|index| format!("t{index} ")).collect();
    let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
    match openai_stream(&refs, Some((3, 4))) {
        Behavior::Stream { chunks, .. } => Behavior::Stream {
            chunks,
            chunk_delay_ms: 100,
        },
        other => other,
    }
}

/// La première ligne annonce l'opération, sous la forme d'un identifiant non vide.
#[tokio::test(flavor = "multi_thread")]
async fn first_line_is_started_with_operation_id() {
    let f = start().await;
    f.server.queue(openai_stream(&["Bon"], None));
    let lines = collect_lines(&chat_json(Some(&f.connection), "Salut")).await;
    assert_eq!(lines[0]["type"], "started", "{lines:?}");
    assert!(!lines[0]["operation"].as_str().unwrap().is_empty());
    assert!(runtime::close().await.contains("\"ok\":true"));
}

/// Les lignes suivent l'ordre `started`, `text`, `usage`, `done`, et le texte se recompose.
#[tokio::test(flavor = "multi_thread")]
async fn lines_follow_started_text_usage_done() {
    let f = start().await;
    f.server
        .queue(openai_stream(&["Bon", "jour"], Some((12, 5))));
    let lines = collect_lines(&chat_json(Some(&f.connection), "Salut")).await;
    let kinds: Vec<&str> = lines.iter().map(|l| l["type"].as_str().unwrap()).collect();
    assert_eq!(
        kinds,
        ["started", "text", "text", "usage", "done"],
        "{lines:?}"
    );
    assert_eq!(lines[1]["text"], "Bon");
    assert_eq!(lines[3]["input_tokens"], 12);
    assert_eq!(lines[3]["output_tokens"], 5);
    assert_eq!(lines[4]["reason"], "end_turn");
    assert!(runtime::close().await.contains("\"ok\":true"));
}

/// Sans connexion désignée, la connexion par défaut sert.
#[tokio::test(flavor = "multi_thread")]
async fn default_connection_serves_when_none_is_named() {
    let f = start().await;
    let set = mira_assembly::api::AiRequest::SetDefaultAiConnection {
        connection: mira_assembly::mira_vocabulary::AiConnectionId::from_raw(f.connection.clone()),
    };
    support::chat_fixture::ask_ai(set).await;
    f.server.queue(openai_stream(&["ok"], None));
    let lines = collect_lines(&chat_json(None, "Salut")).await;
    assert_eq!(lines.last().unwrap()["type"], "done", "{lines:?}");
    assert!(runtime::close().await.contains("\"ok\":true"));
}

/// Quand le flux Dart se ferme, l'opération est annulée et la connexion au fournisseur fermée.
#[tokio::test(flavor = "multi_thread")]
async fn closed_sink_cancels_the_operation() {
    let f = start().await;
    f.server.queue(slow_stream());
    let mut lines: Vec<Value> = Vec::new();
    let request = chat_json(Some(&f.connection), "Salut");
    runtime::pump_chat(&request, |line| {
        lines.push(serde_json::from_str(&line).unwrap());
        lines.len() < 2
    })
    .await;
    assert_eq!(lines.len(), 2, "{lines:?}");
    let operation = lines[0]["operation"].as_str().unwrap();
    let limit = tokio::time::Instant::now() + Duration::from_secs(10);
    while f.server.connections_closed_by_client() < 1 && tokio::time::Instant::now() < limit {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(f.server.connections_closed_by_client() >= 1);
    assert_eq!(operation_status(operation).await["type"], "cancelled");
    assert!(runtime::close().await.contains("\"ok\":true"));
}
