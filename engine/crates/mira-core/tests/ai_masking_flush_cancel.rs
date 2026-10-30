#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Annulation en plein flux : le texte retenu par la restitution sort avant le `Done` d'annulation.

mod ai_chat_support;
mod ai_models_support;
mod support_api;

use ai_chat_support::{answer_of, chat_start_with, collect, connection_with};
use ai_models_support::{ModelsFixture, TestProvider};
use mira_core::api::{OperationRequest, Request};
use mira_testkit::ai::{Behavior, openai_stream};
use mira_vocabulary::{ChatDelta, MaskingSettings, StopReason};

/// Un `[` retenu au moment de l'annulation est rendu avant l'unique `Done { Cancelled }`
#[tokio::test]
async fn held_bracket_is_flushed_before_a_cancelled_done() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let connection = connection_with(&fixture, &MaskingSettings::remote_default());
    let slow = match openai_stream(&["voici [IBAN_", "suite ", "fin"], None) {
        Behavior::Stream { chunks, .. } => Behavior::Stream {
            chunks,
            chunk_delay_ms: 300,
        },
        other => other,
    };
    fixture.server.queue(slow);
    let start = chat_start_with(&connection, "IBAN FR7630006000011234567890189", vec![]);
    let mut stream = fixture.engine.chat_stream(start).await.unwrap();
    let mut deltas = vec![stream.next().await.unwrap(), stream.next().await.unwrap()];
    let cancel = OperationRequest::CancelOperation {
        operation: stream.operation.clone(),
    };
    fixture
        .engine
        .execute(Request::Operation(cancel))
        .await
        .unwrap();
    deltas.extend(collect(&mut stream).await);
    assert_eq!(answer_of(&deltas), "voici [IBAN_");
    let cancelled = ChatDelta::Done {
        reason: StopReason::Cancelled,
    };
    assert_eq!(deltas.last(), Some(&cancelled));
    assert_eq!(deltas.iter().filter(|d| **d == cancelled).count(), 1);
}
