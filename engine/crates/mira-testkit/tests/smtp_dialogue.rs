#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Dialogue SMTP nominal : EHLO, authentification, envoi et réception.

mod smtp_support;

use smtp_support::LIMIT;
use smtp_support::client::SmtpClient;
use smtp_support::flow::{
    authenticated_client, plain_b64, send_message, started_server, xoauth2_b64,
};
use tokio::time::timeout;

/// EHLO annonce les mécanismes d'authentification et les extensions.
#[tokio::test]
async fn ehlo_announces_auth_and_extensions() {
    timeout(LIMIT, async {
        let (server, _) = started_server().await;
        let mut client = SmtpClient::connect(&server.address()).await;
        let reply = client.command("EHLO client.test").await;
        for keyword in ["AUTH PLAIN LOGIN XOAUTH2", "SIZE", "8BITMIME", "PIPELINING"] {
            assert!(reply.contains(keyword), "{keyword} absent de : {reply}");
        }
    })
    .await
    .unwrap();
}

/// Un envoi complet est reçu avec expéditeur, destinataires et données dé-bourrées.
#[tokio::test]
async fn complete_send_is_received_with_unstuffed_data() {
    timeout(LIMIT, async {
        let (server, _) = started_server().await;
        let mut client = authenticated_client(&server).await;
        let body = "Subject: Hi\r\n\r\n.leading dot\r\nlast line\r\n";
        let rcpts = ["bob@example.test", "carol@example.test"];
        let reply = send_message(&mut client, "alice@example.test", &rcpts, body).await;
        assert_eq!(reply, "250 2.0.0 OK queued as mira000001");
        let mails = server.received();
        assert_eq!(mails.len(), 1);
        assert_eq!(mails[0].from, "alice@example.test");
        assert_eq!(mails[0].recipients, rcpts);
        assert_eq!(mails[0].data, body.as_bytes());
    })
    .await
    .unwrap();
}

/// AUTH PLAIN sans réponse initiale déclenche un défi 334 puis réussit.
#[tokio::test]
async fn auth_plain_without_initial_response_gets_challenge() {
    timeout(LIMIT, async {
        let (server, _) = started_server().await;
        let mut client = SmtpClient::connect(&server.address()).await;
        client.command("EHLO client.test").await;
        assert_eq!(client.command("AUTH PLAIN").await, "334");
        let reply = client.command(&plain_b64("alice", "secret")).await;
        assert!(reply.starts_with("235"), "{reply}");
    })
    .await
    .unwrap();
}

/// AUTH LOGIN demande le nom puis le mot de passe en deux étapes.
#[tokio::test]
async fn auth_login_takes_two_steps() {
    timeout(LIMIT, async {
        let (server, _) = started_server().await;
        let mut client = SmtpClient::connect(&server.address()).await;
        client.command("EHLO client.test").await;
        assert_eq!(client.command("AUTH LOGIN").await, "334 VXNlcm5hbWU6");
        assert_eq!(client.command("YWxpY2U=").await, "334 UGFzc3dvcmQ6");
        let reply = client.command("c2VjcmV0").await;
        assert!(reply.starts_with("235"), "{reply}");
    })
    .await
    .unwrap();
}

/// AUTH XOAUTH2 accepte un jeton d'accès enregistré dans les identifiants partagés.
#[tokio::test]
async fn auth_xoauth2_accepts_registered_token() {
    timeout(LIMIT, async {
        let (server, credentials) = started_server().await;
        credentials.add_access_token("alice", "token-1");
        let mut client = SmtpClient::connect(&server.address()).await;
        client.command("EHLO client.test").await;
        let command = format!("AUTH XOAUTH2 {}", xoauth2_b64("alice", "token-1"));
        assert!(client.command(&command).await.starts_with("235"));
        let reply = client.command("MAIL FROM:<alice@example.test>").await;
        assert!(reply.starts_with("250"), "{reply}");
    })
    .await
    .unwrap();
}

/// MAIL FROM avant toute authentification est refusé par 530.
#[tokio::test]
async fn mail_without_authentication_is_refused() {
    timeout(LIMIT, async {
        let (server, _) = started_server().await;
        let mut client = SmtpClient::connect(&server.address()).await;
        client.command("EHLO client.test").await;
        let reply = client.command("MAIL FROM:<alice@example.test>").await;
        assert!(reply.starts_with("530"), "{reply}");
    })
    .await
    .unwrap();
}
