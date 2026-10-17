#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Rafraîchissement, expiration et révocation des jetons OAuth2.

mod oauth_support;

use oauth_support::LIMIT;
use oauth_support::exchange::{address, exchange, refresh};
use oauth_support::flow::{authorize, started};
use oauth_support::http::post_form;
use tokio::time::timeout;

/// Le rafraîchissement émet un nouveau jeton d'accès accepté, sans nouveau jeton de rafraîchissement.
#[tokio::test]
async fn refresh_issues_a_new_accepted_access_token() {
    timeout(LIMIT, async {
        let (server, credentials) = started().await;
        let grant = authorize(&server, "alice", "s").await;
        let first = exchange(&server, &grant).await;
        let reply = refresh(&server, &first.json_field("refresh_token")).await;
        assert_eq!(reply.status, 200, "{}", reply.body);
        let access = reply.json_field("access_token");
        assert_ne!(access, first.json_field("access_token"));
        assert!(credentials.check_access_token("alice", &access));
        assert_eq!(reply.json_field("refresh_token"), "null");
    })
    .await
    .unwrap();
}

/// expire_access_tokens révoque les jetons d'accès mais le rafraîchissement reste possible.
#[tokio::test]
async fn expiring_access_tokens_keeps_refresh_possible() {
    timeout(LIMIT, async {
        let (server, credentials) = started().await;
        let grant = authorize(&server, "alice", "s").await;
        let first = exchange(&server, &grant).await;
        let old = first.json_field("access_token");
        server.expire_access_tokens("alice");
        assert!(!credentials.check_access_token("alice", &old));
        let reply = refresh(&server, &first.json_field("refresh_token")).await;
        assert_eq!(reply.status, 200);
        assert!(credentials.check_access_token("alice", &reply.json_field("access_token")));
    })
    .await
    .unwrap();
}

/// revoke_refresh_tokens rend le rafraîchissement impossible avec invalid_grant.
#[tokio::test]
async fn revoked_refresh_token_gets_invalid_grant() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        let grant = authorize(&server, "alice", "s").await;
        let first = exchange(&server, &grant).await;
        server.revoke_refresh_tokens("alice");
        let reply = refresh(&server, &first.json_field("refresh_token")).await;
        assert_eq!(
            (reply.status, reply.json_field("error")),
            (400, "invalid_grant".into())
        );
    })
    .await
    .unwrap();
}

/// Un jeton de rafraîchissement inconnu reçoit invalid_grant.
#[tokio::test]
async fn unknown_refresh_token_gets_invalid_grant() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        let reply = refresh(&server, "refresh-999999").await;
        assert_eq!(reply.json_field("error"), "invalid_grant");
    })
    .await
    .unwrap();
}

/// Paramètres manquants et type d'autorisation inconnu reçoivent les erreurs prévues.
#[tokio::test]
async fn missing_parameters_and_unknown_grant_type_are_errors() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        let url = address(&server);
        let missing = post_form(&url, "/token", "grant_type=refresh_token").await;
        assert_eq!(missing.json_field("error"), "invalid_request");
        let none = post_form(&url, "/token", "client_id=c").await;
        assert_eq!(none.json_field("error"), "invalid_request");
        let unknown = post_form(&url, "/token", "grant_type=password").await;
        assert_eq!(unknown.json_field("error"), "unsupported_grant_type");
    })
    .await
    .unwrap();
}

/// token_requests compte toutes les requêtes /token et les jetons émis sont tous distincts.
#[tokio::test]
async fn token_requests_are_counted_and_tokens_are_unique() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        let grant = authorize(&server, "alice", "s").await;
        let first = exchange(&server, &grant).await;
        let refresh_token = first.json_field("refresh_token");
        let second = refresh(&server, &refresh_token).await;
        let third = refresh(&server, &refresh_token).await;
        assert_eq!(server.token_requests(), 3);
        let tokens = [
            first.json_field("access_token"),
            second.json_field("access_token"),
            third.json_field("access_token"),
        ];
        assert!(tokens[0] != tokens[1] && tokens[1] != tokens[2] && tokens[0] != tokens[2]);
    })
    .await
    .unwrap();
}
