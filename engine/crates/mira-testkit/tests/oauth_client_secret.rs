#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Un client_secret exigé par le faux serveur : refusé s'il manque, accepté s'il est présenté.

mod oauth_support;

use mira_testkit::oauth::form::build_query;
use oauth_support::LIMIT;
use oauth_support::exchange::{address, exchange};
use oauth_support::flow::{CLIENT_ID, VERIFIER, authorize, started};
use oauth_support::http::post_form;
use tokio::time::timeout;

/// Sans secret présenté, l'échange est refusé avec invalid_client (401)
#[tokio::test]
async fn missing_secret_gets_invalid_client() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        server.require_client_secret("s3cr3t");
        let grant = authorize(&server, "alice", "s").await;
        let reply = exchange(&server, &grant).await;
        assert_eq!(
            (reply.status, reply.json_field("error")),
            (401, "invalid_client".into())
        );
    })
    .await
    .unwrap();
}

/// Avec le bon secret l'échange réussit ; avec un mauvais secret il est refusé
#[tokio::test]
async fn right_secret_is_accepted_and_wrong_is_refused() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        server.require_client_secret("s3cr3t");
        let grant = authorize(&server, "alice", "s").await;
        let body = |secret: &str| {
            build_query(&[
                ("grant_type", "authorization_code"),
                ("code", &grant.code),
                ("redirect_uri", &grant.redirect_uri),
                ("client_id", CLIENT_ID),
                ("code_verifier", VERIFIER),
                ("client_secret", secret),
            ])
        };
        let wrong = post_form(&address(&server), "/token", &body("nope")).await;
        assert_eq!(wrong.status, 401);
        let right = post_form(&address(&server), "/token", &body("s3cr3t")).await;
        assert_eq!(right.status, 200, "{}", right.body);
    })
    .await
    .unwrap();
}
