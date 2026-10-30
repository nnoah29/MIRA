#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Annulation d'une génération : par l'opération, canal plein, flux abandonné, avant le premier octet.

mod ai_chat_support;
mod ai_models_support;
mod support_api;

use std::time::Duration;

use ai_chat_support::{
    collect, loopback_stream, operation_status, wait_cancelled, wait_closed, wait_first_request,
};
use ai_models_support::{ModelsFixture, TestProvider};
use mira_core::api::{OperationRequest, Request};
use mira_core::operations::OperationStatus;
use mira_testkit::ai::{Behavior, openai_stream};
use mira_vocabulary::{ChatDelta, OperationId, StopReason};
use support_api::leaks::events_text;

/// Flux OpenAI de `count` deltas espacés de `delay_ms`
fn slow_stream(count: usize, delay_ms: u64) -> Behavior {
    let texts: Vec<String> = (0..count).map(|index| format!("t{index} ")).collect();
    let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
    match openai_stream(&refs, Some((3, 4))) {
        Behavior::Stream { chunks, .. } => Behavior::Stream {
            chunks,
            chunk_delay_ms: delay_ms,
        },
        other => other,
    }
}

/// Demande l'annulation par le contrat, comme le ferait l'application
async fn cancel(fixture: &ModelsFixture, operation: &OperationId) {
    let request = Request::Operation(OperationRequest::CancelOperation {
        operation: operation.clone(),
    });
    fixture.engine.execute(request).await.unwrap();
}

/// Annuler en plein flux termine par `Done { Cancelled }`, ferme la connexion et clôt l'opération une fois
#[tokio::test]
async fn cancel_operation_ends_with_done_cancelled() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let mut stream = loopback_stream(&fixture, slow_stream(40, 100), "Salut").await;
    assert!(matches!(stream.next().await, Some(ChatDelta::Text { .. })));
    cancel(&fixture, &stream.operation).await;
    let deltas = collect(&mut stream).await;
    assert_eq!(
        deltas.last(),
        Some(&ChatDelta::Done {
            reason: StopReason::Cancelled
        })
    );
    assert!(deltas.len() < 10);
    assert!(wait_cancelled(&fixture.engine, &stream.operation, Duration::from_secs(5)).await);
    assert!(wait_closed(&fixture.server, 1).await);
    let events = events_text(&fixture.engine).await;
    assert_eq!(events.matches("OperationCancelled").count(), 1);
    assert_eq!(events.matches("AiChatFinished").count(), 1);
    assert!(events.contains(r#"outcome: "cancelled""#));
}

/// Un lecteur qui ne lit plus ne bloque pas l'annulation : la tâche se termine en moins d'une seconde
#[tokio::test]
async fn cancel_unblocks_a_full_channel() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let mut stream = loopback_stream(&fixture, slow_stream(100, 0), "Salut").await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    let running = OperationStatus::Running { done: 0, total: 0 };
    assert_eq!(
        operation_status(&fixture.engine, &stream.operation).await,
        running
    );
    cancel(&fixture, &stream.operation).await;
    assert!(wait_cancelled(&fixture.engine, &stream.operation, Duration::from_secs(1)).await);
    let deltas = collect(&mut stream).await;
    assert!(deltas.len() <= 33);
    assert_eq!(
        deltas.last(),
        Some(&ChatDelta::Done {
            reason: StopReason::Cancelled
        })
    );
}

/// Abandonner le flux annule l'opération et ferme la connexion
#[tokio::test]
async fn dropping_the_stream_cancels_the_operation() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let mut stream = loopback_stream(&fixture, slow_stream(40, 100), "Salut").await;
    assert!(matches!(stream.next().await, Some(ChatDelta::Text { .. })));
    let operation = stream.operation.clone();
    drop(stream);
    assert!(wait_cancelled(&fixture.engine, &operation, Duration::from_secs(5)).await);
    assert!(wait_closed(&fixture.server, 1).await);
    let events = events_text(&fixture.engine).await;
    assert_eq!(events.matches("OperationCancelled").count(), 1);
    assert!(events.contains(r#"outcome: "cancelled""#));
}

/// Annuler avant le premier octet de la réponse termine aussitôt par `Done { Cancelled }`
#[tokio::test]
async fn cancel_before_first_byte() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let mut stream = loopback_stream(&fixture, Behavior::Hang, "Salut").await;
    wait_first_request(&fixture.server).await;
    cancel(&fixture, &stream.operation).await;
    let deltas = collect(&mut stream).await;
    assert_eq!(
        deltas,
        vec![ChatDelta::Done {
            reason: StopReason::Cancelled
        }]
    );
    assert!(wait_cancelled(&fixture.engine, &stream.operation, Duration::from_secs(5)).await);
    assert!(wait_closed(&fixture.server, 1).await);
}
