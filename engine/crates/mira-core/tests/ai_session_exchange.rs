#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Session d'abonné : échange du code, client émis, portée exigée, contrôle du jeton d'identité.

mod session_support;

use session_support::connect::{NOW, connect, grant, grant_timed};
use session_support::flow::ISSUER;
use session_support::token_server::TokenServer;

/// L'échange porte le client émis par la redirection (jamais le client dynamique) et la session est complète
#[tokio::test]
async fn exchange_uses_issued_client_id() {
    let server = TokenServer::start().await;
    let ok = |nonce: &str| grant(nonce, "oaiapp_new", ISSUER, "openid scope.required");
    let session = connect(&server, None, "code=C1&client_id=oaiapp_new", &ok)
        .await
        .unwrap();
    assert_eq!(session.client_id, "oaiapp_new");
    assert_eq!(
        (session.access_token.as_str(), session.expires_at),
        ("AT-1", NOW + 3600)
    );
    assert_eq!(session.email.as_deref(), Some("abonne@example.test"));
    let form = server.received().remove(0);
    assert!(
        form.contains("client_id=oaiapp_new") && form.contains("grant_type=authorization_code")
    );
    assert!(
        form.contains("code=C1") && form.contains("code_verifier=") && form.contains("resource=")
    );
    assert!(!form.contains("dynamic_agent_client"));
}

/// Une redirection d'inscription sans client émis échoue avant tout échange
#[tokio::test]
async fn exchange_without_issued_client_is_rejected() {
    let server = TokenServer::start().await;
    let ok = |nonce: &str| grant(nonce, "oaiapp_new", ISSUER, "openid scope.required");
    assert!(connect(&server, None, "code=C1", &ok).await.is_err());
    assert!(server.received().is_empty());
}

/// Sans la portée exigée l'autorisation est refusée
#[tokio::test]
async fn exchange_rejects_missing_scope() {
    let server = TokenServer::start().await;
    let narrow = |nonce: &str| grant(nonce, "oaiapp_new", ISSUER, "openid profile");
    let error = connect(&server, None, "code=C1&client_id=oaiapp_new", &narrow)
        .await
        .unwrap_err();
    assert_eq!(error.code, "authentication.ai_session_incomplete");
}

/// Un jeton d'identité au nonce, à l'émetteur ou au public inattendu est refusé, sans copier son contenu
#[tokio::test]
async fn exchange_rejects_bad_id_token() {
    let bad_nonce = |_: &str| grant("autre", "oaiapp_new", ISSUER, "openid scope.required");
    let bad_iss = |n: &str| {
        grant(
            n,
            "oaiapp_new",
            "https://evil.test",
            "openid scope.required",
        )
    };
    let bad_aud = |n: &str| grant(n, "oaiapp_autre", ISSUER, "openid scope.required");
    let cases: [&dyn Fn(&str) -> String; 3] = [&bad_nonce, &bad_iss, &bad_aud];
    for reply in cases {
        let server = TokenServer::start().await;
        let outcome = connect(&server, None, "code=C1&client_id=oaiapp_new", reply).await;
        let error = outcome.unwrap_err();
        assert!(!format!("{error:?}").contains("AT-1"));
    }
}

/// Une horloge locale en retard de quelques minutes n'empêche pas l'inscription ; un jeton d'identité périmé la refuse
#[tokio::test]
async fn exchange_tolerates_clock_skew_but_not_an_expired_id_token() {
    let server = TokenServer::start().await;
    let slow = |n: &str| {
        grant_timed(
            n,
            "oaiapp_new",
            ISSUER,
            "openid scope.required",
            (NOW + 4000, NOW + 200),
        )
    };
    let skewed = connect(&server, None, "code=C1&client_id=oaiapp_new", &slow).await;
    assert!(skewed.is_ok());
    let old = |n: &str| {
        grant_timed(
            n,
            "oaiapp_new",
            ISSUER,
            "openid scope.required",
            (NOW - 1000, NOW - 4000),
        )
    };
    let expired = connect(&server, None, "code=C1&client_id=oaiapp_new", &old).await;
    assert!(expired.is_err());
}
