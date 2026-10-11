#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Flux d'autorisation OAuth2 : code, PKCE S256, redirection vers le moteur.

mod oauth_support;

use oauth_support::LIMIT;
use oauth_support::exchange::{exchange, exchange_with};
use oauth_support::flow::{authorize, authorize_url, started};
use oauth_support::loopback::Loopback;
use tokio::time::timeout;

/// Le flux complet avec PKCE émet des jetons que le registre d'identifiants accepte.
#[tokio::test]
async fn full_flow_with_pkce_issues_accepted_tokens() {
    timeout(LIMIT, async {
        let (server, credentials) = started().await;
        let grant = authorize(&server, "alice", "st &=%1").await;
        assert_eq!(grant.state, "st &=%1");
        let reply = exchange(&server, &grant).await;
        assert_eq!(reply.status, 200, "{}", reply.body);
        assert_eq!(reply.json_field("token_type"), "Bearer");
        assert_eq!(reply.json_field("expires_in"), "3600");
        assert!(reply.headers.contains("Cache-Control: no-store"));
        let access = reply.json_field("access_token");
        assert!(credentials.check_access_token("alice", &access));
        assert!(!credentials.check_access_token("bob", &access));
        assert!(!reply.json_field("refresh_token").is_empty());
    })
    .await
    .unwrap();
}

/// deny redirige avec access_denied et l'état d'origine.
#[tokio::test]
async fn deny_redirects_with_access_denied_and_state() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        let loopback = Loopback::bind().await;
        let url = authorize_url(&server, &loopback.redirect_uri(), "abc");
        let pending = loopback.expect_redirect();
        server.deny(&url).await.unwrap();
        let target = pending.await.unwrap();
        assert_eq!(target, "/callback?error=access_denied&state=abc");
    })
    .await
    .unwrap();
}

/// Un mauvais vérificateur est refusé et consomme le code.
#[tokio::test]
async fn wrong_verifier_is_refused_and_burns_the_code() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        let grant = authorize(&server, "alice", "s").await;
        let wrong = "x".repeat(43);
        let reply = exchange_with(&server, &grant, &wrong).await;
        assert_eq!(
            (reply.status, reply.json_field("error")),
            (400, "invalid_grant".into())
        );
        let retry = exchange(&server, &grant).await;
        assert_eq!(retry.json_field("error"), "invalid_grant");
    })
    .await
    .unwrap();
}

/// Un code déjà échangé ne peut pas l'être une seconde fois.
#[tokio::test]
async fn reused_code_is_refused() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        let grant = authorize(&server, "alice", "s").await;
        assert_eq!(exchange(&server, &grant).await.status, 200);
        let reply = exchange(&server, &grant).await;
        assert_eq!(
            (reply.status, reply.json_field("error")),
            (400, "invalid_grant".into())
        );
    })
    .await
    .unwrap();
}

/// Un redirect_uri différent de celui de la requête d'autorisation est refusé.
#[tokio::test]
async fn mismatched_redirect_uri_is_refused() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        let mut grant = authorize(&server, "alice", "s").await;
        grant.redirect_uri = "http://127.0.0.1:1/other".to_string();
        let reply = exchange(&server, &grant).await;
        assert_eq!(reply.json_field("error"), "invalid_grant");
    })
    .await
    .unwrap();
}

/// approve refuse une méthode PKCE autre que S256 et une redirection hors boucle locale.
#[tokio::test]
async fn approve_rejects_plain_pkce_and_non_loopback_redirects() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        let url = authorize_url(&server, "http://127.0.0.1:9/cb", "s");
        let plain = url.replace("code_challenge_method=S256", "code_challenge_method=plain");
        assert!(server.approve(&plain, "alice").await.is_err());
        let remote = authorize_url(&server, "http://example.com:9/cb", "s");
        assert!(server.approve(&remote, "alice").await.is_err());
    })
    .await
    .unwrap();
}
