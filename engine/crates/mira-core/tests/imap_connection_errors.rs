#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Refus du clair, serveur injoignable ou muet, TLS impossible, LOGIN de repli.

mod support_imap;

use std::time::Duration;

use mira_core::imap::{ImapClient, ImapOptions};
use mira_testkit::imap::CapabilitySet;
use mira_vocabulary::{Security, ServerEndpoint};
use support_imap::env::{endpoint, local_endpoint, start};
use support_imap::logins::password_login;
use support_imap::scripted::scripted;
use support_imap::within;

/// Le clair vers un hôte qui n'est pas la boucle locale est refusé avant tout échange réseau
#[tokio::test]
async fn plaintext_to_a_remote_host_is_refused() {
    within(async {
        let remote = ServerEndpoint {
            host: "mail.example.net".to_string(),
            port: 143,
            security: Security::Plaintext,
        };
        let error = ImapClient::connect(&remote, &password_login("secret"))
            .await
            .unwrap_err();
        assert_eq!(error.code, "tls.plaintext_refused");
        assert!(!format!("{error:?}").contains("secret"));
    })
    .await;
}

/// `localhost` est reconnu comme boucle locale et le clair y est accepté
#[tokio::test]
async fn localhost_name_is_accepted_in_plaintext() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let mut target = endpoint(&env);
        target.host = "localhost".to_string();
        let client = ImapClient::connect(&target, &password_login("secret")).await;
        client.unwrap().logout().await.unwrap();
    })
    .await;
}

/// Un port fermé donne network.unreachable
#[tokio::test]
async fn closed_port_is_unreachable() {
    within(async {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let target = local_endpoint(port, Security::Plaintext);
        let error = ImapClient::connect(&target, &password_login("secret"))
            .await
            .unwrap_err();
        assert_eq!(error.code, "network.unreachable");
        assert!(error.retryable);
    })
    .await;
}

/// Un serveur muet donne network.timeout dans le délai injecté
#[tokio::test]
async fn silent_server_times_out() {
    within(async {
        let (port, task) = scripted("", &[]).await;
        let options = ImapOptions {
            timeout: Duration::from_millis(300),
        };
        let target = local_endpoint(port, Security::Plaintext);
        let error = ImapClient::connect_with(&target, &password_login("secret"), options)
            .await
            .unwrap_err();
        assert_eq!(error.code, "network.timeout");
        task.abort();
    })
    .await;
}

/// Un serveur qui ne parle pas TLS donne tls.handshake
#[tokio::test]
async fn tls_against_a_plaintext_server_fails_the_handshake() {
    within(async {
        let (port, task) = scripted("* OK pas du TLS\r\n", &[]).await;
        let target = local_endpoint(port, Security::Tls);
        let error = ImapClient::connect(&target, &password_login("secret"))
            .await
            .unwrap_err();
        assert_eq!(error.category.as_str(), "tls");
        task.abort();
    })
    .await;
}

/// Sans AUTH=PLAIN annoncé, la connexion passe par LOGIN avec les arguments cités
#[tokio::test]
async fn login_command_is_used_without_plain_mechanism() {
    within(async {
        let greeting = "* OK [CAPABILITY IMAP4rev1] pret\r\n";
        let done = "{tag} OK [CAPABILITY IMAP4rev1 IDLE] connecte\r\n";
        let (port, task) = scripted(greeting, &[done, "{tag} OK au revoir\r\n"]).await;
        let target = local_endpoint(port, Security::Plaintext);
        let client = ImapClient::connect(&target, &password_login("se\"cret"))
            .await
            .unwrap();
        assert!(client.capabilities().iter().any(|c| c == "IDLE"));
        client.logout().await.unwrap();
        let received = task.await.unwrap();
        assert_eq!(received[0], "A0001 LOGIN \"alice\" \"se\\\"cret\"");
        assert_eq!(received[1], "A0002 LOGOUT");
    })
    .await;
}
