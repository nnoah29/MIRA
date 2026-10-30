#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Journal des envois : lecteur parti en cours de route, coupure du flux, adresses qui imitent la boucle locale.

mod ai_chat_support;
mod ai_models_support;
#[allow(dead_code)]
mod support_ai_context;
mod support_api;

use ai_chat_support::{
    add_chat_connection, allow_kinds, chat_start_with, collect, give_consent, journal_rows,
    loopback_stream, wait_journal_outcome,
};
use ai_models_support::{ModelsFixture, TestProvider};
use mira_testkit::ai::{Behavior, openai_stream};
use mira_vocabulary::error::codes::{incompatible_state, server};
use mira_vocabulary::{ChatDelta, ContentKind};
use support_ai_context::part;

/// Génération lente de `count` morceaux, un toutes les `delay_ms`
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

/// Le lecteur qui part en pleine génération laisse une ligne `cancelled`
#[tokio::test]
async fn reader_dropped_mid_generation_completes_the_row_as_cancelled() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let mut stream = loopback_stream(&fixture, slow_stream(40, 100), "Salut").await;
    assert!(matches!(stream.next().await, Some(ChatDelta::Text { .. })));
    let operation = stream.operation.clone();
    drop(stream);
    assert_eq!(
        wait_journal_outcome(&fixture, &operation).await,
        "cancelled"
    );
    assert_eq!(journal_rows(&fixture).len(), 1);
}

/// Une coupure du flux après du texte complète la ligne avec le code de l'erreur
#[tokio::test]
async fn mid_stream_failure_completes_the_row_with_the_error_code() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let chunk = b"data: {\"choices\":[{\"delta\":{\"content\":\"Bon\"}}]}\n\n".to_vec();
    let behavior = Behavior::DropAfter {
        chunks: vec![chunk],
    };
    let mut stream = loopback_stream(&fixture, behavior, "Salut").await;
    collect(&mut stream).await;
    let outcome = wait_journal_outcome(&fixture, &stream.operation).await;
    assert_eq!(outcome, server::AI_UNAVAILABLE.0);
}

/// Une adresse qui ressemble à la boucle locale sans l'être est distante : type non autorisé refusé, rien envoyé
#[tokio::test]
async fn lookalike_loopback_urls_are_treated_as_remote() {
    for url in [
        "http://localhost.evil.com/v1",
        "http://127.0.0.1@evil.com/v1",
    ] {
        let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
        let connection = add_chat_connection(&fixture, "conn-1", Some(url.to_string()), Some("m"));
        fixture.store_key("conn-1", "sk-test");
        give_consent(&fixture, &connection);
        allow_kinds(&fixture, &connection, &[ContentKind::MailList]);
        let parts = vec![part(ContentKind::MailMessage, "Sujet", "corps")];
        let refused = fixture
            .engine
            .chat_stream(chat_start_with(&connection, "q", parts))
            .await;
        let code = refused.unwrap_err().code;
        assert_eq!(code, incompatible_state::AI_CONTENT_NOT_ALLOWED.0, "{url}");
        assert!(fixture.server.requests().is_empty() && journal_rows(&fixture).is_empty());
    }
}
