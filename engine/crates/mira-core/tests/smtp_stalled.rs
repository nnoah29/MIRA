#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Client SMTP : un serveur qui se tait après la connexion donne une issue incertaine, jamais un délai passager.

mod support_smtp;

use std::time::Duration;

use mira_core::smtp::send_within;
use mira_vocabulary::{Security, ServerEndpoint};
use support_smtp::LIMIT;
use support_smtp::server::{PASSWORD, outgoing, password_login};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::time::timeout;

/// Serveur qui accepte la connexion, salue puis ne répond plus jamais
async fn stalling_endpoint(greet: bool) -> ServerEndpoint {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        if greet {
            stream.write_all(b"220 ready\r\n").await.unwrap();
        }
        tokio::time::sleep(Duration::from_secs(60)).await;
    });
    ServerEndpoint {
        host: "127.0.0.1".to_string(),
        port,
        security: Security::Plaintext,
    }
}

/// Un silence après le message de bienvenue dépasse le délai : network.connection_lost, non réessayable
#[tokio::test]
async fn regression_stall_after_greeting_is_connection_lost_not_timeout() {
    timeout(LIMIT, async {
        let target = stalling_endpoint(true).await;
        let login = password_login(PASSWORD);
        let limit = Duration::from_millis(500);
        let error = send_within(&target, &login, &outgoing(), limit)
            .await
            .unwrap_err();
        assert_eq!(error.code, "network.connection_lost");
        assert!(!error.retryable);
    })
    .await
    .unwrap();
}

/// Un serveur muet dès la connexion donne le même verdict incertain
#[tokio::test]
async fn regression_silent_server_is_connection_lost_not_timeout() {
    timeout(LIMIT, async {
        let target = stalling_endpoint(false).await;
        let login = password_login(PASSWORD);
        let limit = Duration::from_millis(500);
        let error = send_within(&target, &login, &outgoing(), limit)
            .await
            .unwrap_err();
        assert_eq!(error.code, "network.connection_lost");
        assert!(!error.retryable);
    })
    .await
    .unwrap();
}
