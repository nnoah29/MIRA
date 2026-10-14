#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Client SMTP : une erreur du client qui n'est pas un refus d'identifiants n'est pas une authentification.

mod support_smtp;

use mira_core::smtp::send_message;
use mira_vocabulary::{Security, ServerEndpoint};
use support_smtp::LIMIT;
use support_smtp::server::{PASSWORD, outgoing, password_login};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::time::timeout;

/// Serveur dont l'extension AUTH n'annonce aucun mécanisme que le client sache employer
async fn endpoint_without_usable_auth() -> ServerEndpoint {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let (reader, mut writer) = stream.into_split();
        writer.write_all(b"220 ready\r\n").await.unwrap();
        let mut lines = BufReader::new(reader).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let reply: &[u8] = if line.starts_with("EHLO") {
                b"250-hi\r\n250 AUTH CRAM-MD5\r\n"
            } else {
                b"221 bye\r\n"
            };
            writer.write_all(reply).await.unwrap();
        }
    });
    ServerEndpoint {
        host: "127.0.0.1".to_string(),
        port,
        security: Security::Plaintext,
    }
}

/// Aucun mécanisme d'authentification commun : server.protocol, pas un refus d'identifiants
#[tokio::test]
async fn regression_missing_auth_mechanism_is_a_protocol_error_not_authentication() {
    timeout(LIMIT, async {
        let target = endpoint_without_usable_auth().await;
        let error = send_message(&target, &password_login(PASSWORD), &outgoing())
            .await
            .unwrap_err();
        assert_eq!(error.code, "server.protocol");
        assert!(!error.retryable);
    })
    .await
    .unwrap();
}
