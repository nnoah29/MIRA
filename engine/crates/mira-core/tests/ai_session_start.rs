#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Session d'abonné : secret au trousseau, adresse d'inscription et de reconnexion.

mod session_support;

use mira_core::auth::session::AiSession;
use mira_core::ports::SecretValue;
use session_support::browser::query_of;
use session_support::flow::{HOST, login, start};

/// Une session de test
fn session() -> AiSession {
    AiSession {
        client_id: "oaiapp_issued".into(),
        host_id: HOST.into(),
        access_token: "AT-SECRET".into(),
        refresh_token: "RT-SECRET".into(),
        expires_at: 42,
        email: Some("abonne@example.test".into()),
    }
}

/// La session fait l'aller-retour par le secret, et un secret quelconque est refusé sans être recopié
#[test]
fn session_roundtrips_through_secret() {
    let back = AiSession::from_secret(&session().to_secret()).unwrap();
    assert_eq!(back.client_id, "oaiapp_issued");
    assert_eq!(back.refresh_token, "RT-SECRET");
    let error = AiSession::from_secret(&SecretValue::new("sk-pas-une-session")).unwrap_err();
    assert!(!format!("{error:?}").contains("sk-pas"));
}

/// `Debug` n'affiche ni jeton ni courriel
#[test]
fn session_debug_hides_tokens() {
    let text = format!("{:?}", session());
    for secret in ["AT-SECRET", "RT-SECRET", "abonne@example.test"] {
        assert!(!text.contains(secret), "{text}");
    }
}

/// L'inscription demande le client dynamique, nomme l'application et porte l'hôte, le nonce, la ressource et toutes les portées
#[tokio::test]
async fn start_registration_url_has_dynamic_client_and_host() {
    let flow = start(&login("http://127.0.0.1:9/oauth/token"), None).await;
    let query = query_of(flow.authorize_url());
    assert!(
        flow.authorize_url()
            .starts_with("https://auth.provider.test/api/accounts/authorize?")
    );
    assert_eq!(query["client_id"], "dynamic_agent_client");
    assert_eq!(query["agent_name_hint"], "MIRA");
    assert_eq!(query["ext_agent_host_id"], HOST);
    assert_eq!(query["response_type"], "code");
    assert_eq!(query["code_challenge_method"], "S256");
    assert_eq!(query["resource"], "https://api.provider.test/v1");
    assert_eq!(query["scope"], "openid profile scope.required");
    assert!(query["redirect_uri"].starts_with("http://127.0.0.1:"));
    assert!(query["redirect_uri"].ends_with("/auth/callback"));
    assert!(!query["nonce"].is_empty() && !query["state"].is_empty());
}

/// La reconnexion reprend le client émis et l'hôte, ajoute l'indice de courriel et n'envoie plus le nom
#[tokio::test]
async fn start_reconnect_url_uses_issued_client_and_no_name() {
    let flow = start(&login("http://127.0.0.1:9/oauth/token"), Some(&session())).await;
    let query = query_of(flow.authorize_url());
    assert_eq!(query["client_id"], "oaiapp_issued");
    assert_eq!(query["ext_agent_host_id"], HOST);
    assert_eq!(query["login_hint"], "abonne@example.test");
    assert!(!query.contains_key("agent_name_hint"));
}
