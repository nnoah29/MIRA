#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Consentement : rien ne part vers un service distant sans accord ; la boucle locale est exemptée.

mod ai_chat_support;
mod ai_models_support;
mod support_api;

use ai_chat_support::{add_chat_connection, chat_start, collect, give_consent};
use ai_models_support::{ModelsFixture, TestProvider};
use mira_testkit::ai::openai_stream;
use mira_vocabulary::error::codes::{incompatible_state, invalid_input};
use mira_vocabulary::{ChatDelta, StopReason};

const REMOTE: &str = "http://ai.provider.test/v1";

/// Connexion distante sans consentement : refus local, aucune requête émise
#[tokio::test]
async fn remote_connection_without_consent_is_refused_and_no_request_sent() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let connection = add_chat_connection(&fixture, "conn-1", Some(REMOTE.to_string()), Some("m"));
    fixture.store_key("conn-1", "sk-test");
    fixture.server.queue(openai_stream(&["ok"], None));
    let error = fixture
        .engine
        .chat_stream(chat_start(&connection, "Salut"))
        .await
        .unwrap_err();
    assert_eq!(error.code, incompatible_state::AI_CONSENT_REQUIRED.0);
    assert!(fixture.server.requests().is_empty());
}

/// Le consentement est vérifié avant la clé ; une fois donné, c'est la clé absente qui est signalée
#[tokio::test]
async fn consent_is_checked_before_the_key() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let connection = add_chat_connection(&fixture, "conn-1", Some(REMOTE.to_string()), Some("m"));
    let error = fixture
        .engine
        .chat_stream(chat_start(&connection, "Salut"))
        .await
        .unwrap_err();
    assert_eq!(error.code, incompatible_state::AI_CONSENT_REQUIRED.0);
    give_consent(&fixture, &connection);
    let error = fixture
        .engine
        .chat_stream(chat_start(&connection, "Salut"))
        .await
        .unwrap_err();
    assert_eq!(error.code, incompatible_state::AI_KEY_MISSING.0);
}

/// Avec consentement et clé, les vérifications locales passent et la génération démarre
#[tokio::test]
async fn consent_given_allows_the_request() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let connection = add_chat_connection(&fixture, "conn-1", Some(REMOTE.to_string()), Some("m"));
    fixture.store_key("conn-1", "sk-test");
    give_consent(&fixture, &connection);
    let mut stream = fixture
        .engine
        .chat_stream(chat_start(&connection, "Salut"))
        .await
        .unwrap();
    let refused = StopReason::Failed {
        code: invalid_input::AI_URL_INVALID.0.to_string(),
        retry_after_secs: None,
    };
    assert_eq!(
        collect(&mut stream).await,
        vec![ChatDelta::Done { reason: refused }]
    );
}

/// Boucle locale : ni consentement ni clé exigés, la génération aboutit
#[tokio::test]
async fn loopback_connection_is_exempt() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let base = Some(fixture.server.base_url().replace("127.0.0.1", "localhost"));
    let connection = add_chat_connection(&fixture, "conn-1", base, Some("m"));
    fixture.server.queue(openai_stream(&["ok"], None));
    let mut stream = fixture
        .engine
        .chat_stream(chat_start(&connection, "Salut"))
        .await
        .unwrap();
    let deltas = collect(&mut stream).await;
    assert_eq!(
        deltas.last(),
        Some(&ChatDelta::Done {
            reason: StopReason::EndTurn
        })
    );
    assert_eq!(fixture.server.requests().len(), 1);
}
