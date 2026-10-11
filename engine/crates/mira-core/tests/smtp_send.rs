#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Client SMTP contre le faux serveur : remise, authentification, XOAUTH2.

mod support_smtp;

use mira_core::ports::SecretValue;
use mira_core::smtp::{SmtpCredential, SmtpLogin, send_message};
use support_smtp::LIMIT;
use support_smtp::server::{PASSWORD, USER, endpoint, outgoing, password_login, started};
use tokio::time::timeout;

/// Un envoi complet arrive chez le serveur avec enveloppe, Bcc hors en-têtes et Message-ID
#[tokio::test]
async fn full_delivery_reaches_the_server_with_envelope_and_headers() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        let message = outgoing();
        send_message(&endpoint(&server), &password_login(PASSWORD), &message)
            .await
            .unwrap();
        let received = server.received();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].from, "alice@example.org");
        assert_eq!(received[0].recipients, message.recipients);
        assert!(
            received[0]
                .recipients
                .contains(&"dave@example.org".to_string())
        );
        let text = String::from_utf8_lossy(&received[0].data).into_owned();
        assert!(!text.contains("dave@example.org"), "{text}");
        assert!(text.contains("Message-ID: <m1@mira.local>"), "{text}");
        assert!(text.contains("Salut Bob"), "{text}");
    })
    .await
    .unwrap();
}

/// Un mot de passe refusé donne authentication.rejected, sans secret dans l'erreur
#[tokio::test]
async fn wrong_password_is_rejected_without_leaking_it() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        let wrong = "mauvais-Mot-De-Passe-77";
        let login = password_login(wrong);
        let error = send_message(&endpoint(&server), &login, &outgoing())
            .await
            .unwrap_err();
        assert_eq!(error.code, "authentication.rejected");
        assert!(!error.retryable);
        let shown = format!("{error:?} {error} {login:?}");
        assert!(
            !shown.contains(wrong) && !shown.contains(PASSWORD),
            "{shown}"
        );
        assert!(server.received().is_empty());
    })
    .await
    .unwrap();
}

/// XOAUTH2 : un jeton connu passe, un jeton révoqué donne authentication.token_expired
#[tokio::test]
async fn xoauth2_accepts_known_token_and_reports_expired_one() {
    timeout(LIMIT, async {
        let (server, credentials) = started().await;
        credentials.add_access_token(USER, "tok-access-123");
        let login = |token: &str| SmtpLogin {
            user: USER.to_string(),
            credential: SmtpCredential::XOAuth2(SecretValue::new(token)),
        };
        send_message(&endpoint(&server), &login("tok-access-123"), &outgoing())
            .await
            .unwrap();
        assert_eq!(server.received().len(), 1);
        credentials.revoke_access_token("tok-access-123");
        let error = send_message(&endpoint(&server), &login("tok-access-123"), &outgoing())
            .await
            .unwrap_err();
        assert_eq!(error.code, "authentication.token_expired");
        assert!(!format!("{error:?}").contains("tok-access-123"));
    })
    .await
    .unwrap();
}

/// Un destinataire ou un expéditeur d'enveloppe invalide est refusé avant toute connexion
#[tokio::test]
async fn invalid_envelope_addresses_are_refused() {
    timeout(LIMIT, async {
        let (server, _) = started().await;
        let mut message = outgoing();
        message.recipients = vec!["pas une adresse".to_string()];
        let error = send_message(&endpoint(&server), &password_login(PASSWORD), &message)
            .await
            .unwrap_err();
        assert_eq!(error.code, "invalid_input.address");
        let mut empty = outgoing();
        empty.recipients.clear();
        let error = send_message(&endpoint(&server), &password_login(PASSWORD), &empty)
            .await
            .unwrap_err();
        assert_eq!(error.code, "invalid_input.address");
    })
    .await
    .unwrap();
}
