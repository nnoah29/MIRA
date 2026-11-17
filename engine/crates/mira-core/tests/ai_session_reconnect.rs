#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Session d'abonné : la reconnexion garde le client connu et refuse un client différent.

mod session_support;

use mira_core::auth::session::AiSession;
use session_support::connect::{connect, grant};
use session_support::flow::{HOST, ISSUER};
use session_support::token_server::TokenServer;

/// Une reconnexion garde le client connu ; un client différent dans la redirection est refusé
#[tokio::test]
async fn reconnect_keeps_known_client_and_rejects_a_different_one() {
    let known = AiSession {
        client_id: "oaiapp_known".into(),
        host_id: HOST.into(),
        access_token: String::new(),
        refresh_token: String::new(),
        expires_at: 0,
        email: None,
    };
    let server = TokenServer::start().await;
    let ok = |nonce: &str| grant(nonce, "oaiapp_known", ISSUER, "openid scope.required");
    let same = connect(&server, Some(&known), "code=C1", &ok)
        .await
        .unwrap();
    assert_eq!(same.client_id, "oaiapp_known");
    let other = connect(&server, Some(&known), "code=C1&client_id=oaiapp_other", &ok).await;
    assert!(other.is_err());
}
