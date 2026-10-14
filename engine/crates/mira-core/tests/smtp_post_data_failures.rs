#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Client SMTP : seules les pannes prouvées avant DATA sont passagères ; un silence à la fin de DATA est incertain.

mod support_smtp;

use std::time::Duration;

use mira_core::smtp::send_within;
use mira_vocabulary::{Security, ServerEndpoint};
use support_smtp::LIMIT;
use support_smtp::server::{PASSWORD, endpoint, outgoing, password_login, started};
use tokio::net::TcpListener;
use tokio::time::timeout;

const OVERALL: Duration = Duration::from_millis(600);

/// Un silence à la fin de DATA alors que le serveur a gardé le message : issue incertaine, non réessayable
#[tokio::test]
async fn stall_at_end_of_data_is_connection_lost_and_not_retryable() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        server.stall_after_data_once();
        let login = password_login(PASSWORD);
        let error = send_within(&endpoint(&server), &login, &outgoing(), OVERALL)
            .await
            .unwrap_err();
        assert_eq!(error.code, "network.connection_lost");
        assert!(!error.retryable);
        assert_eq!(server.received().len(), 1);
    })
    .await
    .unwrap();
}

/// Un refus 4xx avant DATA reste passager et le serveur n'a rien gardé
#[tokio::test]
async fn transient_refusal_before_data_stays_retryable() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        server.fail_next("451 4.7.1 Greylisted");
        let login = password_login(PASSWORD);
        let error = send_within(&endpoint(&server), &login, &outgoing(), OVERALL)
            .await
            .unwrap_err();
        assert_eq!(error.code, "server.rate_limited");
        assert!(error.retryable);
        assert!(server.received().is_empty());
    })
    .await
    .unwrap();
}

/// Une connexion TCP refusée reste network.unreachable, réessayable
#[tokio::test]
async fn refused_tcp_connection_stays_unreachable_and_retryable() {
    timeout(LIMIT, async {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let target = ServerEndpoint {
            host: "127.0.0.1".to_string(),
            port,
            security: Security::Plaintext,
        };
        let login = password_login(PASSWORD);
        let error = send_within(&target, &login, &outgoing(), OVERALL)
            .await
            .unwrap_err();
        assert_eq!(error.code, "network.unreachable");
        assert!(error.retryable);
    })
    .await
    .unwrap();
}
