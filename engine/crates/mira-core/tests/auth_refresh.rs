#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Rafraîchissement silencieux des jetons (7.7, S10) et envoi SMTP avec XOAUTH2.

mod support_auth;
mod support_smtp;

use mira_core::auth::refresh_tokens;
use mira_core::smtp::{SmtpCredential, SmtpLogin, send_message};
use mira_testkit::Credentials;
use mira_testkit::smtp::FakeSmtpServer;
use support_auth::flow::{approved_flow, clock};
use support_auth::{CLIENT_ID, LIMIT, USER, started_oauth};
use support_smtp::server::{endpoint, outgoing};
use tokio::time::timeout;

/// Le rafraîchissement donne un nouveau jeton d'accès accepté et une nouvelle expiration
#[tokio::test]
async fn refresh_returns_a_new_accepted_access_token() {
    timeout(LIMIT, async {
        let (server, credentials) = started_oauth().await;
        let (flow, tokens) = approved_flow(&server, 21).await;
        let refresh = tokens.refresh_token.clone().unwrap();
        server.expire_access_tokens(USER);
        let fresh = refresh_tokens(flow.token_url(), CLIENT_ID, &refresh, &clock())
            .await
            .unwrap();
        assert_ne!(fresh.access_token.expose(), tokens.access_token.expose());
        assert!(credentials.check_access_token(USER, fresh.access_token.expose()));
        assert!(fresh.refresh_token.is_none());
        assert_eq!(fresh.expires_at.unwrap().as_millis(), 1_000_000 + 3_600_000);
    })
    .await
    .unwrap();
}

/// Un jeton de rafraîchissement révoqué donne authentication.token_revoked (compte à réauthentifier)
#[tokio::test]
async fn revoked_refresh_token_requires_reauthentication() {
    timeout(LIMIT, async {
        let (server, _) = started_oauth().await;
        let (flow, tokens) = approved_flow(&server, 22).await;
        let refresh = tokens.refresh_token.clone().unwrap();
        server.revoke_refresh_tokens(USER);
        let error = refresh_tokens(flow.token_url(), CLIENT_ID, &refresh, &clock())
            .await
            .unwrap_err();
        assert_eq!(error.code, "authentication.token_revoked");
        assert!(!error.retryable);
        assert!(!format!("{error:?}").contains(refresh.expose()));
    })
    .await
    .unwrap();
}

/// Un client inconnu ou un jeton inconnu est refusé de la même façon
#[tokio::test]
async fn unknown_refresh_token_is_refused() {
    timeout(LIMIT, async {
        let (server, _) = started_oauth().await;
        let unknown = mira_core::ports::SecretValue::new("refresh-999999");
        let error = refresh_tokens(&server.token_url(), CLIENT_ID, &unknown, &clock())
            .await
            .unwrap_err();
        assert_eq!(error.code, "authentication.token_revoked");
    })
    .await
    .unwrap();
}

/// Bout en bout : jeton émis par le faux OAuth, rejeté une fois expiré, puis rafraîchi et accepté en SMTP
#[tokio::test]
async fn smtp_xoauth2_works_with_issued_then_refreshed_tokens() {
    timeout(LIMIT, async {
        let (oauth, credentials) = started_oauth().await;
        let smtp = FakeSmtpServer::start(Credentials::clone(&credentials))
            .await
            .unwrap();
        let (flow, tokens) = approved_flow(&oauth, 23).await;
        let login = |token: &mira_core::ports::SecretValue| SmtpLogin {
            user: USER.to_string(),
            credential: SmtpCredential::XOAuth2(token.clone()),
        };
        send_message(&endpoint(&smtp), &login(&tokens.access_token), &outgoing())
            .await
            .unwrap();
        oauth.expire_access_tokens(USER);
        let expired =
            send_message(&endpoint(&smtp), &login(&tokens.access_token), &outgoing()).await;
        assert_eq!(expired.unwrap_err().code, "authentication.token_expired");
        let refresh = tokens.refresh_token.clone().unwrap();
        let fresh = refresh_tokens(flow.token_url(), CLIENT_ID, &refresh, &clock())
            .await
            .unwrap();
        send_message(&endpoint(&smtp), &login(&fresh.access_token), &outgoing())
            .await
            .unwrap();
        assert_eq!(smtp.received().len(), 2);
    })
    .await
    .unwrap();
}
