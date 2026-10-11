#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Connexion et authentification du client IMAP contre le faux serveur.

mod support_imap;

use std::time::Duration;

use mira_core::imap::ImapClient;
use mira_testkit::imap::CapabilitySet;
use support_imap::env::{endpoint, start};
use support_imap::logins::{password_login, token_login};
use support_imap::within;

/// Mot de passe accepté : capacités après authentification, puis déconnexion propre
#[tokio::test]
async fn connect_with_password_then_logout() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let client = ImapClient::connect(&endpoint(&env), &password_login("secret"))
            .await
            .unwrap();
        assert!(client.capabilities().iter().any(|c| c == "IDLE"));
        assert!(!client.capabilities().iter().any(|c| c.starts_with("AUTH=")));
        assert_eq!(env.server.active_connections(), 1);
        client.logout().await.unwrap();
        for _ in 0..100 {
            if env.server.active_connections() == 0 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        assert_eq!(env.server.active_connections(), 0);
    })
    .await;
}

/// Mot de passe refusé : erreur d'authentification sans le mot de passe nulle part
#[tokio::test]
async fn wrong_password_is_rejected_without_leaking_it() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let login = password_login("hunter2-faux");
        let error = ImapClient::connect(&endpoint(&env), &login)
            .await
            .unwrap_err();
        assert_eq!(error.code, "authentication.rejected");
        assert!(!error.retryable);
        let shown = format!("{error:?} {error} {login:?}");
        assert!(!shown.contains("hunter2-faux"), "{shown}");
    })
    .await;
}

/// XOAUTH2 : un jeton valide connecte, un jeton révoqué demande un rafraîchissement
#[tokio::test]
async fn xoauth2_token_accepted_then_revoked() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let client = ImapClient::connect(&endpoint(&env), &token_login("tok-1"))
            .await
            .unwrap();
        client.logout().await.unwrap();
        env.credentials.revoke_access_token("tok-1");
        let login = token_login("tok-1");
        let error = ImapClient::connect(&endpoint(&env), &login)
            .await
            .unwrap_err();
        assert_eq!(error.code, "authentication.token_expired");
        assert!(error.retryable);
        assert!(!format!("{error:?} {login:?}").contains("tok-1"));
    })
    .await;
}

/// Le mécanisme PLAIN est utilisé quand le serveur l'annonce
#[tokio::test]
async fn plain_mechanism_is_used_when_announced() {
    within(async {
        let env = start(CapabilitySet::minimal()).await;
        let client = ImapClient::connect(&endpoint(&env), &password_login("secret"))
            .await
            .unwrap();
        client.logout().await.unwrap();
        let log = env.store.command_log();
        assert!(
            log.iter()
                .any(|line| line.starts_with("AUTHENTICATE PLAIN"))
        );
    })
    .await;
}
