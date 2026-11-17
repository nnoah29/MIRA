#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Discussion avec une session d'abonné : jeton frais, renouvelé avant l'appel, échecs, concurrence.

mod ai_chat_support;
mod ai_models_support;
mod session_chat_support;
mod session_support;
mod support_api;

use ai_chat_support::{chat_start, collect};
use ai_models_support::ModelsFixture;
use session_chat_support::responses::{responses_stream, text_of};
use session_chat_support::{NOW, provider, stored, with_session};
use session_support::token_server::TokenServer;

const REFRESHED: &str = r#"{"access_token":"AT-NEW","refresh_token":"RT-NEW","expires_in":3600}"#;

/// Un jeton valide sert tel quel, sans appel au point de jeton, et le flux Responses se décode
#[tokio::test]
async fn fresh_token_is_used_without_refresh() {
    let token = TokenServer::start().await;
    let fixture = ModelsFixture::start(provider(&token)).await;
    let connection = with_session(&fixture, NOW + 3600);
    fixture.server.queue(responses_stream());
    let mut stream = fixture
        .engine
        .chat_stream(chat_start(&connection, "Salut"))
        .await
        .unwrap();
    assert_eq!(text_of(&collect(&mut stream).await), "Bonjour");
    let request = &fixture.server.requests()[0];
    assert_eq!(request.path, "/responses");
    assert_eq!(request.header("authorization"), Some("Bearer AT-OLD"));
    assert!(token.received().is_empty());
}

/// Un jeton proche de l'expiration est renouvelé, la nouvelle session est rangée et son jeton sert à l'appel
#[tokio::test]
async fn expiring_token_is_refreshed_and_stored_before_use() {
    let token = TokenServer::start().await;
    token.push(200, REFRESHED);
    let fixture = ModelsFixture::start(provider(&token)).await;
    let connection = with_session(&fixture, NOW + 60);
    fixture.server.queue(responses_stream());
    let mut stream = fixture
        .engine
        .chat_stream(chat_start(&connection, "Salut"))
        .await
        .unwrap();
    collect(&mut stream).await;
    let request = &fixture.server.requests()[0];
    assert_eq!(request.header("authorization"), Some("Bearer AT-NEW"));
    assert!(!request.body.contains("RT-"));
    let kept = stored(&fixture);
    assert_eq!(
        (kept.access_token.as_str(), kept.refresh_token.as_str()),
        ("AT-NEW", "RT-NEW")
    );
    assert_eq!(kept.client_id, "oaiapp_issued");
}

/// Deux discussions simultanées avec un jeton expiré ne rafraîchissent qu'une fois
#[tokio::test]
async fn concurrent_chats_refresh_once() {
    let token = TokenServer::start().await;
    token.push(200, REFRESHED);
    let fixture = ModelsFixture::start(provider(&token)).await;
    let connection = with_session(&fixture, NOW - 10);
    fixture.server.queue(responses_stream());
    let (first, second) = tokio::join!(
        fixture.engine.chat_stream(chat_start(&connection, "A")),
        fixture.engine.chat_stream(chat_start(&connection, "B")),
    );
    collect(&mut first.unwrap()).await;
    collect(&mut second.unwrap()).await;
    assert_eq!(token.received().len(), 1);
    for request in fixture.server.requests() {
        assert_eq!(request.header("authorization"), Some("Bearer AT-NEW"));
    }
}

/// Un refus permanent donne `ai_session_expired` et laisse la session telle quelle
#[tokio::test]
async fn permanent_refresh_error_is_session_expired_and_keeps_session() {
    let token = TokenServer::start().await;
    token.push(400, r#"{"error":"invalid_grant"}"#);
    let fixture = ModelsFixture::start(provider(&token)).await;
    let connection = with_session(&fixture, NOW - 10);
    let error = fixture
        .engine
        .chat_stream(chat_start(&connection, "Salut"))
        .await
        .unwrap_err();
    assert_eq!(error.code, "authentication.ai_session_expired");
    assert_eq!(stored(&fixture).refresh_token, "RT-OLD");
    assert!(fixture.server.requests().is_empty());
}

/// Un échec passager donne `ai_unavailable`, sans rien effacer
#[tokio::test]
async fn transient_refresh_error_is_unavailable_and_keeps_session() {
    let token = TokenServer::start().await;
    token.push(503, "{}");
    let fixture = ModelsFixture::start(provider(&token)).await;
    let connection = with_session(&fixture, NOW - 10);
    let error = fixture
        .engine
        .chat_stream(chat_start(&connection, "Salut"))
        .await
        .unwrap_err();
    assert_eq!(error.code, "server.ai_unavailable");
    assert_eq!(stored(&fixture).refresh_token, "RT-OLD");
}
