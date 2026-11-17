#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Discussion avec une session d'abonné : moteur prêt et session rangée.

pub mod failing_store;
pub mod responses;

use mira_core::auth::session::AiSession;
use mira_core::ports::SecretStore;
use mira_vocabulary::{AiConnection, AiConnectionId, AiSecretRef, ProfileId, WireFormat};

use crate::ai_chat_support::add_chat_connection;
use crate::ai_models_support::{ModelsFixture, TestProvider};
use crate::session_support::flow::{HOST, login};
use crate::session_support::token_server::TokenServer;

pub const NOW: i64 = 1_700_000_000;

/// Fournisseur de test au format Responses dont la connexion de session vise ce point de jeton
pub fn provider(token: &TokenServer) -> TestProvider {
    let mut provider = TestProvider::new("test-provider");
    provider.format = WireFormat::OpenAiResponses;
    provider.session_login = Some(login(&token.url));
    provider
}

/// Session qui expire à `expires_at`
pub fn session(expires_at: i64) -> AiSession {
    AiSession {
        client_id: "oaiapp_issued".into(),
        host_id: HOST.into(),
        access_token: "AT-OLD".into(),
        refresh_token: "RT-OLD".into(),
        expires_at,
        email: None,
    }
}

/// Connexion `conn-1` et session rangée au trousseau
pub fn with_session(fixture: &ModelsFixture, expires_at: i64) -> AiConnection {
    let base = Some(fixture.server.base_url());
    let connection = add_chat_connection(fixture, "conn-1", base, Some("model-a"));
    put_session(fixture, &session(expires_at));
    connection
}

/// Range cette session pour `conn-1`
pub fn put_session(fixture: &ModelsFixture, session: &AiSession) {
    fixture
        .secrets
        .put(&location(), &session.to_secret())
        .unwrap();
}

/// Session actuellement rangée pour `conn-1`
pub fn stored(fixture: &ModelsFixture) -> AiSession {
    AiSession::from_secret(&fixture.secrets.get(&location()).unwrap().unwrap()).unwrap()
}

/// Emplacement du secret de `conn-1`
fn location() -> AiSecretRef {
    AiSecretRef {
        profile: ProfileId::from_raw("default"),
        connection: AiConnectionId::from_raw("conn-1"),
    }
}
