#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Vérifications locales avant tout envoi : connexion, connexion par défaut, fournisseur.

mod ai_chat_support;
mod ai_models_support;
mod support_api;

use ai_chat_support::{add_chat_connection, collect, make_default, user_request};
use ai_models_support::{ModelsFixture, TestProvider};
use mira_core::api::ChatStart;
use mira_testkit::ai::openai_stream;
use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{AiConnectionId, ChatDelta, StopReason};

/// Démarrage sans connexion désignée
fn default_start() -> ChatStart {
    ChatStart {
        connection: None,
        request: user_request("Salut"),
    }
}

/// Une connexion inconnue est `not_found.ai_connection`
#[tokio::test]
async fn unknown_connection_is_not_found() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    let start = ChatStart {
        connection: Some(AiConnectionId::from_raw("absente")),
        request: user_request("Salut"),
    };
    let error = fixture.engine.chat_stream(start).await.unwrap_err();
    assert_eq!(error.code, not_found::AI_CONNECTION.0);
}

/// Sans connexion par défaut, demander la connexion par défaut est `not_found.ai_connection`
#[tokio::test]
async fn missing_default_connection_is_not_found() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    add_chat_connection(
        &fixture,
        "conn-1",
        Some(fixture.server.base_url()),
        Some("m"),
    );
    let error = fixture
        .engine
        .chat_stream(default_start())
        .await
        .unwrap_err();
    assert_eq!(error.code, not_found::AI_CONNECTION.0);
}

/// Sans connexion désignée, la connexion par défaut sert
#[tokio::test]
async fn default_connection_is_used_when_none_given() {
    let fixture = ModelsFixture::start(TestProvider::new("test-provider")).await;
    add_chat_connection(
        &fixture,
        "conn-0",
        Some("http://ai.provider.test".to_string()),
        None,
    );
    let chosen = add_chat_connection(
        &fixture,
        "conn-1",
        Some(fixture.server.base_url()),
        Some("m"),
    );
    make_default(&fixture, &chosen);
    fixture.server.queue(openai_stream(&["ok"], None));
    let mut stream = fixture.engine.chat_stream(default_start()).await.unwrap();
    let deltas = collect(&mut stream).await;
    assert_eq!(
        deltas.last(),
        Some(&ChatDelta::Done {
            reason: StopReason::EndTurn
        })
    );
}

/// Un fournisseur absent du registre est `not_found.ai_provider`, sans requête
#[tokio::test]
async fn unknown_provider_is_not_found() {
    let fixture = ModelsFixture::start(TestProvider::new("autre-fournisseur")).await;
    let connection = add_chat_connection(
        &fixture,
        "conn-1",
        Some(fixture.server.base_url()),
        Some("m"),
    );
    let start = ChatStart {
        connection: Some(connection.id),
        request: user_request("Salut"),
    };
    let error = fixture.engine.chat_stream(start).await.unwrap_err();
    assert_eq!(error.code, not_found::AI_PROVIDER.0);
    assert!(fixture.server.requests().is_empty());
}
