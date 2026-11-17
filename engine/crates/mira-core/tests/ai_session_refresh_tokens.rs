#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Session d'abonné : rafraîchissement, jetons de remplacement, échecs permanents et passagers.

mod session_support;

use mira_core::auth::session::{AiSession, SessionRefreshError, refresh_session};
use session_support::flow::{HOST, login};
use session_support::token_server::TokenServer;

const NOW: i64 = 5_000;

/// Session proche de l'expiration
fn old_session() -> AiSession {
    AiSession {
        client_id: "oaiapp_issued".into(),
        host_id: HOST.into(),
        access_token: "AT-OLD".into(),
        refresh_token: "RT-OLD".into(),
        expires_at: 1,
        email: Some("abonne@example.test".into()),
    }
}

/// Le rafraîchissement remplace les deux jetons et l'échéance, garde le client, l'hôte et le courriel
#[tokio::test]
async fn refresh_replaces_both_tokens() {
    let server = TokenServer::start().await;
    server.push(
        200,
        r#"{"access_token":"AT-NEW","refresh_token":"RT-NEW","expires_in":3600}"#,
    );
    let fresh = refresh_session(&login(&server.url), &old_session(), NOW)
        .await
        .unwrap();
    assert_eq!(
        (fresh.access_token.as_str(), fresh.refresh_token.as_str()),
        ("AT-NEW", "RT-NEW")
    );
    assert_eq!(fresh.expires_at, NOW + 3600);
    assert_eq!(
        (fresh.client_id.as_str(), fresh.email.as_deref()),
        ("oaiapp_issued", Some("abonne@example.test"))
    );
    let form = server.received().remove(0);
    assert!(form.contains("grant_type=refresh_token") && form.contains("refresh_token=RT-OLD"));
    assert!(form.contains("client_id=oaiapp_issued") && !form.contains("scope="));
}

/// Les refus du service sont permanents, jusqu'à la réponse d'erreur illisible
#[tokio::test]
async fn refresh_permanent_errors() {
    for (status, body) in [
        (400, r#"{"error":"invalid_grant"}"#),
        (401, r#"{"error":"refresh_token_reused"}"#),
        (401, ""),
    ] {
        let server = TokenServer::start().await;
        server.push(status, body);
        let outcome = refresh_session(&login(&server.url), &old_session(), NOW).await;
        assert_eq!(
            outcome.unwrap_err(),
            SessionRefreshError::Permanent,
            "{status}"
        );
    }
}

/// Réseau, 5xx, 408, 429 et succès sans jeton de remplacement sont passagers
#[tokio::test]
async fn refresh_transient_errors() {
    let unreachable = login("http://127.0.0.1:9/oauth/token");
    let down = refresh_session(&unreachable, &old_session(), NOW).await;
    assert_eq!(down.unwrap_err(), SessionRefreshError::Transient);
    let cases = [
        (503, "{}"),
        (408, "{}"),
        (429, "{}"),
        (200, r#"{"access_token":"AT-NEW"}"#),
    ];
    for (status, body) in cases {
        let server = TokenServer::start().await;
        server.push(status, body);
        let outcome = refresh_session(&login(&server.url), &old_session(), NOW).await;
        assert_eq!(
            outcome.unwrap_err(),
            SessionRefreshError::Transient,
            "{status}"
        );
    }
}
