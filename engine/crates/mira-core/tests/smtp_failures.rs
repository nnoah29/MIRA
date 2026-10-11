#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Client SMTP : conversion des pannes (4xx, 552, 5xx, coupure après DATA, clair refusé).

mod support_smtp;

use mira_core::smtp::send_message;
use mira_vocabulary::{Security, ServerEndpoint};
use support_smtp::LIMIT;
use support_smtp::server::{PASSWORD, endpoint, outgoing, password_login, started};
use tokio::net::TcpListener;
use tokio::time::timeout;

/// 552 devient server.message_too_large, définitif
#[tokio::test]
async fn too_large_message_maps_to_message_too_large() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        server.fail_next("552 5.3.4 Message too big");
        let error = send_message(&endpoint(&server), &password_login(PASSWORD), &outgoing())
            .await
            .unwrap_err();
        assert_eq!(error.code, "server.message_too_large");
        assert!(!error.retryable);
        assert!(server.received().is_empty());
    })
    .await
    .unwrap();
}

/// 4xx est réessayable (limitation de débit ou refus transitoire), 5xx est définitif
#[tokio::test]
async fn transient_and_permanent_rejections_are_told_apart() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        let target = endpoint(&server);
        let login = password_login(PASSWORD);
        for (reply, code, retryable) in [
            ("451 4.7.1 Greylisted", "server.rate_limited", true),
            ("452 4.3.1 Insufficient storage", "server.rejected", true),
            ("550 5.7.1 Rejected", "server.rejected", false),
        ] {
            server.fail_next(reply);
            let error = send_message(&target, &login, &outgoing())
                .await
                .unwrap_err();
            assert_eq!(
                (error.code.as_str(), error.retryable),
                (code, retryable),
                "{reply}"
            );
        }
        send_message(&target, &login, &outgoing()).await.unwrap();
    })
    .await
    .unwrap();
}

/// Une coupure après DATA donne network.connection_lost non réessayable (issue incertaine)
#[tokio::test]
async fn connection_cut_after_data_is_an_uncertain_outcome() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        server.drop_after_data_once();
        let error = send_message(&endpoint(&server), &password_login(PASSWORD), &outgoing())
            .await
            .unwrap_err();
        assert_eq!(error.code, "network.connection_lost");
        assert!(!error.retryable);
        assert_eq!(server.received().len(), 1);
    })
    .await
    .unwrap();
}

/// Un port fermé donne network.unreachable, réessayable
#[tokio::test]
async fn closed_port_is_unreachable_and_retryable() {
    timeout(LIMIT, async {
        let port = TcpListener::bind("127.0.0.1:0")
            .await
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let target = ServerEndpoint {
            host: "127.0.0.1".to_string(),
            port,
            security: Security::Plaintext,
        };
        let error = send_message(&target, &password_login(PASSWORD), &outgoing())
            .await
            .unwrap_err();
        assert_eq!(error.code, "network.unreachable");
        assert!(error.retryable);
    })
    .await
    .unwrap();
}

/// Le clair vers un hôte non local est refusé sans connexion (AG-03) ; la boucle locale passe
#[tokio::test]
async fn plaintext_is_refused_outside_the_loopback() {
    timeout(LIMIT, async {
        for host in ["mail.example.org", "10.0.0.5", "127.0.0.1.example.org"] {
            let target = ServerEndpoint {
                host: host.to_string(),
                port: 25,
                security: Security::Plaintext,
            };
            let error = send_message(&target, &password_login(PASSWORD), &outgoing())
                .await
                .unwrap_err();
            assert_eq!(error.code, "tls.plaintext_refused", "{host}");
        }
        let (server, _) = started().await;
        send_message(&endpoint(&server), &password_login(PASSWORD), &outgoing())
            .await
            .unwrap();
    })
    .await
    .unwrap();
}

/// STARTTLS exigé mais non proposé par le serveur : tls.handshake, rien n'est envoyé
#[tokio::test]
async fn starttls_without_server_support_fails_with_tls_error() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        let mut target = endpoint(&server);
        target.security = Security::StartTls;
        let error = send_message(&target, &password_login(PASSWORD), &outgoing())
            .await
            .unwrap_err();
        assert_eq!(error.code, "tls.handshake");
        assert!(server.received().is_empty());
    })
    .await
    .unwrap();
}
