#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Refus d'authentification SMTP : mauvais secrets, jeton révoqué, mécanisme inconnu.

mod smtp_support;

use smtp_support::LIMIT;
use smtp_support::client::SmtpClient;
use smtp_support::flow::{plain_b64, started_server, xoauth2_b64};
use tokio::time::timeout;

/// Un mot de passe incorrect en PLAIN reçoit 535.
#[tokio::test]
async fn wrong_plain_password_is_refused() {
    timeout(LIMIT, async {
        let (server, _) = started_server().await;
        let mut client = SmtpClient::connect(&server.address()).await;
        client.command("EHLO client.test").await;
        let reply = client
            .command(&format!("AUTH PLAIN {}", plain_b64("alice", "wrong")))
            .await;
        assert!(reply.starts_with("535"), "{reply}");
    })
    .await
    .unwrap();
}

/// Un mot de passe incorrect en LOGIN reçoit 535 puis MAIL reste interdit.
#[tokio::test]
async fn wrong_login_password_is_refused() {
    timeout(LIMIT, async {
        let (server, _) = started_server().await;
        let mut client = SmtpClient::connect(&server.address()).await;
        client.command("EHLO client.test").await;
        client.command("AUTH LOGIN").await;
        client.command("YWxpY2U=").await;
        assert!(client.command("d3Jvbmc=").await.starts_with("535"));
        let reply = client.command("MAIL FROM:<alice@example.test>").await;
        assert!(reply.starts_with("530"), "{reply}");
    })
    .await
    .unwrap();
}

/// Un jeton XOAUTH2 révoqué est refusé par 535.
#[tokio::test]
async fn revoked_xoauth2_token_is_refused() {
    timeout(LIMIT, async {
        let (server, credentials) = started_server().await;
        credentials.add_access_token("alice", "token-1");
        credentials.revoke_access_token("token-1");
        let mut client = SmtpClient::connect(&server.address()).await;
        client.command("EHLO client.test").await;
        let command = format!("AUTH XOAUTH2 {}", xoauth2_b64("alice", "token-1"));
        assert!(client.command(&command).await.starts_with("535"));
    })
    .await
    .unwrap();
}

/// Un jeton valide pour un autre utilisateur est refusé par 535.
#[tokio::test]
async fn token_of_another_user_is_refused() {
    timeout(LIMIT, async {
        let (server, credentials) = started_server().await;
        credentials.add_access_token("bob", "token-bob");
        let mut client = SmtpClient::connect(&server.address()).await;
        client.command("EHLO client.test").await;
        let command = format!("AUTH XOAUTH2 {}", xoauth2_b64("alice", "token-bob"));
        assert!(client.command(&command).await.starts_with("535"));
    })
    .await
    .unwrap();
}

/// Un mécanisme inconnu reçoit 504 et un base64 invalide reçoit 535.
#[tokio::test]
async fn unknown_mechanism_and_bad_base64_are_refused() {
    timeout(LIMIT, async {
        let (server, _) = started_server().await;
        let mut client = SmtpClient::connect(&server.address()).await;
        client.command("EHLO client.test").await;
        assert!(client.command("AUTH CRAM-MD5").await.starts_with("504"));
        assert!(
            client
                .command("AUTH PLAIN !!!not-base64")
                .await
                .starts_with("535")
        );
    })
    .await
    .unwrap();
}
