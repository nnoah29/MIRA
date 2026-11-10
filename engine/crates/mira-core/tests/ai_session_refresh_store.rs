#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Discussion avec une session d'abonné : si le trousseau refuse d'écrire, aucun appel n'est fait avec le jeton neuf.

mod ai_chat_support;
mod ai_models_support;
mod session_chat_support;
mod session_support;
mod support_api;

use std::sync::Arc;

use ai_chat_support::chat_start;
use ai_models_support::ModelsFixture;
use session_chat_support::failing_store::FailingPut;
use session_chat_support::{NOW, provider, stored, with_session};
use session_support::token_server::TokenServer;

/// Écriture refusée après le rafraîchissement : erreur, ancienne session lisible, aucune requête d'inférence
#[tokio::test]
async fn store_failure_means_no_inference_call() {
    let token = TokenServer::start().await;
    token.push(
        200,
        r#"{"access_token":"AT-NEW","refresh_token":"RT-NEW","expires_in":3600}"#,
    );
    let wrap = |inner| Arc::new(FailingPut(inner)) as Arc<dyn mira_core::ports::SecretStore>;
    let fixture = ModelsFixture::start_with_store(vec![provider(&token)], wrap).await;
    let connection = with_session(&fixture, NOW - 10);
    let outcome = fixture
        .engine
        .chat_stream(chat_start(&connection, "Salut"))
        .await;
    assert!(outcome.is_err());
    assert!(fixture.server.requests().is_empty());
    assert_eq!(stored(&fixture).refresh_token, "RT-OLD");
}
