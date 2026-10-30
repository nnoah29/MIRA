#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Génération en flux : les échecs du service arrivent comme dernier delta `Done { Failed }`.

mod ai_chat_support;
mod ai_models_support;
mod support_api;

use std::time::Duration;

use ai_chat_support::{add_chat_connection, chat_start, collect, wait_status};
use ai_models_support::{ModelsFixture, TestProvider};
use mira_core::operations::OperationStatus;
use mira_testkit::ai::Behavior;
use mira_vocabulary::error::codes::{authentication, server};
use mira_vocabulary::{ChatDelta, StopReason};

/// Delta de fin en échec avec ce code et ce délai
fn failed(code: &str, retry_after_secs: Option<u64>) -> ChatDelta {
    ChatDelta::Done {
        reason: StopReason::Failed {
            code: code.to_string(),
            retry_after_secs,
        },
    }
}

/// Réponse complète de ce statut avec ces en-têtes
fn status(code: u16, headers: &[(&str, &str)]) -> Behavior {
    Behavior::Status {
        code,
        headers: headers
            .iter()
            .map(|(n, v)| (n.to_string(), v.to_string()))
            .collect(),
        body: r#"{"error":{"message":"refus"}}"#.to_string(),
    }
}

/// Un 401 se termine par `Failed { authentication.ai_key_rejected }` et l'opération échoue
#[tokio::test]
async fn http_401_ends_with_done_failed_key_rejected() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let base = Some(fixture.server.base_url());
    let connection = add_chat_connection(&fixture, "conn-1", base, Some("model-a"));
    fixture.server.queue(status(401, &[]));
    let mut stream = fixture
        .engine
        .chat_stream(chat_start(&connection, "Salut"))
        .await
        .unwrap();
    let code = authentication::AI_KEY_REJECTED.0;
    assert_eq!(collect(&mut stream).await, vec![failed(code, None)]);
    let expected = OperationStatus::Failed {
        code: code.to_string(),
    };
    let ok = wait_status(
        &fixture.engine,
        &stream.operation,
        &expected,
        Duration::from_secs(5),
    );
    assert!(ok.await);
}

/// Un 429 porte le délai d'attente annoncé par le service
#[tokio::test]
async fn http_429_carries_retry_after_secs() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let base = Some(fixture.server.base_url());
    let connection = add_chat_connection(&fixture, "conn-1", base, Some("model-a"));
    fixture.server.queue(status(429, &[("Retry-After", "7")]));
    let mut stream = fixture
        .engine
        .chat_stream(chat_start(&connection, "Salut"))
        .await
        .unwrap();
    let deltas = collect(&mut stream).await;
    assert_eq!(deltas, vec![failed(server::AI_RATE_LIMITED.0, Some(7))]);
}

/// Une coupure en plein flux garde le texte reçu puis se termine par `Failed { server.ai_unavailable }`
#[tokio::test]
async fn mid_stream_drop_ends_with_done_failed_unavailable() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let base = Some(fixture.server.base_url());
    let connection = add_chat_connection(&fixture, "conn-1", base, Some("model-a"));
    let chunk = b"data: {\"choices\":[{\"delta\":{\"content\":\"Bon\"}}]}\n\n".to_vec();
    fixture.server.queue(Behavior::DropAfter {
        chunks: vec![chunk],
    });
    let mut stream = fixture
        .engine
        .chat_stream(chat_start(&connection, "Salut"))
        .await
        .unwrap();
    let deltas = collect(&mut stream).await;
    let text = ChatDelta::Text {
        text: "Bon".to_string(),
    };
    assert_eq!(deltas, vec![text, failed(server::AI_UNAVAILABLE.0, None)]);
}
