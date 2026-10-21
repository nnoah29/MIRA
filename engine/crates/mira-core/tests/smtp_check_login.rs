#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Test de connexion SMTP (AG-14) : authentification réelle, aucun message envoyé.

mod support_smtp;

use std::time::Duration;

use mira_core::smtp::check_smtp_login;
use support_smtp::server::{PASSWORD, endpoint, password_login, started};
use tokio::time::timeout;

const LIMIT: Duration = Duration::from_secs(20);

/// Un mot de passe accepté réussit, sans rien envoyer
#[tokio::test]
async fn accepted_password_passes_without_sending_anything() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        check_smtp_login(&endpoint(&server), &password_login(PASSWORD))
            .await
            .unwrap();
        assert!(server.received().is_empty());
    })
    .await
    .unwrap();
}

/// Un mot de passe refusé donne authentication.rejected : le test s'authentifie vraiment
#[tokio::test]
async fn rejected_password_is_an_authentication_error() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        let error = check_smtp_login(&endpoint(&server), &password_login("mauvais"))
            .await
            .unwrap_err();
        assert_eq!(error.code, "authentication.rejected");
    })
    .await
    .unwrap();
}

/// Un serveur qui n'écoute pas donne network.unreachable, réessayable
#[tokio::test]
async fn unreachable_server_is_a_retryable_network_error() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        let mut target = endpoint(&server);
        target.port = 1;
        let error = check_smtp_login(&target, &password_login(PASSWORD))
            .await
            .unwrap_err();
        assert_eq!(error.code, "network.unreachable");
        assert!(error.retryable);
    })
    .await
    .unwrap();
}
