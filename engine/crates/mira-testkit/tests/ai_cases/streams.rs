//! Flux SSE des formats OpenAI Chat et Anthropic Messages.

use std::time::{Duration, Instant};

use mira_testkit::ai::{Behavior, FakeAiServer, anthropic_stream, openai_stream};

use crate::ai_support::client::{exchange, post_request};

/// Le flux OpenAI émet les deltas, l'arrêt, l'usage puis `[DONE]` en SSE découpé en morceaux.
#[tokio::test]
async fn serves_openai_stream_as_sse() {
    let server = FakeAiServer::start().await.unwrap();
    server.queue(openai_stream(&["Bon", "jour \"toi\"\n"], Some((11, 7))));
    let reply = exchange(
        &server.base_url(),
        &post_request("/v1/chat/completions", &[], "{}"),
    )
    .await;
    assert_eq!(reply.status, 200);
    assert!(reply.complete);
    assert_eq!(reply.header("content-type"), Some("text/event-stream"));
    let text = reply.text();
    let events: Vec<&str> = text.split("\n\n").filter(|e| !e.is_empty()).collect();
    assert_eq!(events.len(), 5, "{text}");
    assert_eq!(
        events[0],
        r#"data: {"choices":[{"delta":{"content":"Bon"}}]}"#
    );
    assert!(events[1].contains(r#"\"toi\"\n"#), "{}", events[1]);
    assert!(events[2].contains(r#""finish_reason":"stop""#));
    assert!(events[3].contains(r#""choices":[]"#) && events[3].contains(r#""prompt_tokens":11"#));
    assert!(events[3].contains(r#""completion_tokens":7"#));
    assert_eq!(events[4], "data: [DONE]");
    server.stop().await;
}

/// Sans usage, le flux OpenAI n'émet aucun bloc d'usage.
#[tokio::test]
async fn openai_stream_without_usage_has_no_usage_event() {
    let server = FakeAiServer::start().await.unwrap();
    server.queue(openai_stream(&["a"], None));
    let reply = exchange(
        &server.base_url(),
        &post_request("/v1/chat/completions", &[], "{}"),
    )
    .await;
    assert!(!reply.text().contains("usage"));
    assert!(reply.text().ends_with("data: [DONE]\n\n"));
    server.stop().await;
}

/// Le flux Anthropic suit message_start, content_block_delta, message_delta, message_stop.
#[tokio::test]
async fn serves_anthropic_stream_as_sse() {
    let server = FakeAiServer::start().await.unwrap();
    server.queue(anthropic_stream(&["Bon", "jour"], Some((25, 15))));
    let reply = exchange(&server.base_url(), &post_request("/v1/messages", &[], "{}")).await;
    assert_eq!(reply.status, 200);
    assert_eq!(reply.header("content-type"), Some("text/event-stream"));
    let text = reply.text();
    let names: Vec<&str> = text
        .lines()
        .filter_map(|l| l.strip_prefix("event: "))
        .collect();
    let expected = [
        "message_start",
        "content_block_delta",
        "content_block_delta",
        "message_delta",
        "message_stop",
    ];
    assert_eq!(names, expected, "{text}");
    assert!(text.contains(r#""input_tokens":25"#));
    assert!(text.contains(r#""output_tokens":15"#));
    assert!(text.contains(r#""text":"Bon""#) && text.contains(r#""type":"text_delta""#));
    assert!(text.contains(r#""stop_reason":"end_turn""#));
    server.stop().await;
}

/// chunk_delay_ms espacie les morceaux, sans retarder les en-têtes.
#[tokio::test]
async fn chunk_delay_spaces_the_chunks() {
    let server = FakeAiServer::start().await.unwrap();
    let chunks = vec![
        b"data: 1\n\n".to_vec(),
        b"data: 2\n\n".to_vec(),
        b"data: 3\n\n".to_vec(),
    ];
    server.queue(Behavior::Stream {
        chunks,
        chunk_delay_ms: 150,
    });
    let started = Instant::now();
    let reply = exchange(&server.base_url(), &post_request("/x", &[], "{}")).await;
    assert!(started.elapsed() >= Duration::from_millis(280));
    assert_eq!(reply.text(), "data: 1\n\ndata: 2\n\ndata: 3\n\n");
    server.stop().await;
}
