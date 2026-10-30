#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Autorisation par type de contenu : consentement d'abord, types ensuite, clé enfin ; boucle locale exemptée.

mod ai_chat_support;
mod ai_models_support;
#[allow(dead_code)]
mod support_ai_context;
mod support_api;

use ai_chat_support::{
    add_chat_connection, allow_kinds, chat_start, chat_start_with, collect, give_consent,
    journal_rows,
};
use ai_models_support::{ModelsFixture, TestProvider};
use mira_testkit::ai::openai_stream;
use mira_vocabulary::error::codes::{incompatible_state, invalid_input};
use mira_vocabulary::{ChatDelta, ContentKind, StopReason};
use support_ai_context::part;

const REMOTE: &str = "http://ai.provider.test/v1";

/// Fixture et connexion distante sans consentement ni type autorisé
async fn remote() -> (ModelsFixture, mira_vocabulary::AiConnection) {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let connection = add_chat_connection(&fixture, "conn-1", Some(REMOTE.to_string()), Some("m"));
    (fixture, connection)
}

/// Un type non autorisé est refusé avant tout accès réseau et ne laisse aucune ligne de journal
#[tokio::test]
async fn remote_connection_refuses_a_kind_not_allowed_and_sends_nothing() {
    let (fixture, connection) = remote().await;
    fixture.store_key("conn-1", "sk-test");
    give_consent(&fixture, &connection);
    allow_kinds(&fixture, &connection, &[ContentKind::MailMessage]);
    fixture.server.queue(openai_stream(&["ok"], None));
    let parts = vec![part(ContentKind::MailList, "Boîte", "liste")];
    let refused = fixture
        .engine
        .chat_stream(chat_start_with(&connection, "q", parts))
        .await;
    assert_eq!(
        refused.unwrap_err().code,
        incompatible_state::AI_CONTENT_NOT_ALLOWED.0
    );
    assert!(fixture.server.requests().is_empty());
    assert!(journal_rows(&fixture).is_empty());
}

/// Un type autorisé passe les vérifications locales ; une discussion sans partie n'est jamais restreinte
#[tokio::test]
async fn allowed_kind_goes_through() {
    let (fixture, connection) = remote().await;
    fixture.store_key("conn-1", "sk-test");
    give_consent(&fixture, &connection);
    allow_kinds(&fixture, &connection, &[ContentKind::MailMessage]);
    let parts = vec![part(ContentKind::MailMessage, "Sujet", "corps")];
    let start = chat_start_with(&connection, "q", parts);
    let mut stream = fixture.engine.chat_stream(start).await.unwrap();
    let plaintext = StopReason::Failed {
        code: invalid_input::AI_URL_INVALID.0.to_string(),
        retry_after_secs: None,
    };
    assert_eq!(
        collect(&mut stream).await,
        vec![ChatDelta::Done { reason: plaintext }]
    );
    assert_eq!(journal_rows(&fixture).len(), 1);
    allow_kinds(&fixture, &connection, &[]);
    assert!(
        fixture
            .engine
            .chat_stream(chat_start(&connection, "q"))
            .await
            .is_ok()
    );
}

/// Vers la boucle locale, aucun type n'a besoin d'être autorisé
#[tokio::test]
async fn loopback_connection_is_exempt_from_kinds() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let base = Some(fixture.server.base_url());
    let connection = add_chat_connection(&fixture, "conn-1", base, Some("m"));
    fixture.server.queue(openai_stream(&["ok"], None));
    let parts = vec![part(ContentKind::Attachment, "Pièce", "CONTENU-PIECE")];
    let start = chat_start_with(&connection, "q", parts);
    let mut stream = fixture.engine.chat_stream(start).await.unwrap();
    let deltas = collect(&mut stream).await;
    assert!(deltas.contains(&ChatDelta::Text {
        text: "ok".to_string()
    }));
    assert!(fixture.server.requests()[0].body.contains("CONTENU-PIECE"));
}

/// Sans consentement, c'est le consentement qui est réclamé, même si le type n'est pas autorisé non plus
#[tokio::test]
async fn consent_is_checked_before_kinds() {
    let (fixture, connection) = remote().await;
    let parts = vec![part(ContentKind::MailMessage, "Sujet", "corps")];
    let refused = fixture
        .engine
        .chat_stream(chat_start_with(&connection, "q", parts))
        .await;
    assert_eq!(
        refused.unwrap_err().code,
        incompatible_state::AI_CONSENT_REQUIRED.0
    );
}

/// Les types sont vérifiés avant la clé : sans clé, un type refusé reste signalé en premier
#[tokio::test]
async fn kinds_are_checked_before_the_key() {
    let (fixture, connection) = remote().await;
    give_consent(&fixture, &connection);
    let parts = || vec![part(ContentKind::MailMessage, "Sujet", "corps")];
    let refused = fixture
        .engine
        .chat_stream(chat_start_with(&connection, "q", parts()))
        .await;
    assert_eq!(
        refused.unwrap_err().code,
        incompatible_state::AI_CONTENT_NOT_ALLOWED.0
    );
    let plain = fixture
        .engine
        .chat_stream(chat_start(&connection, "q"))
        .await;
    assert_eq!(
        plain.unwrap_err().code,
        incompatible_state::AI_KEY_MISSING.0
    );
    allow_kinds(&fixture, &connection, &[ContentKind::MailMessage]);
    let allowed = fixture
        .engine
        .chat_stream(chat_start_with(&connection, "q", parts()))
        .await;
    assert_eq!(
        allowed.unwrap_err().code,
        incompatible_state::AI_KEY_MISSING.0
    );
}
