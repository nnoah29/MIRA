#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Flux OAuth2 PKCE (15.2) : URL d'autorisation, redirection en boucle locale, refus et délai.

mod support_auth;

use std::time::Duration;

use mira_core::auth::pkce::challenge_for;
use mira_core::auth::{FlowPhase, start_flow, wait_for_redirect};
use support_auth::browser::{get, query_value};
use support_auth::{CLIENT_ID, LIMIT, USER, oauth_settings, seeded_rng, started_oauth};
use tokio::time::timeout;

/// Le flux complet reçoit le code approuvé par le faux navigateur, sans quitter la boucle locale
#[tokio::test]
async fn approved_flow_delivers_the_code_on_the_loopback_listener() {
    timeout(LIMIT, async {
        let (server, _) = started_oauth().await;
        let mut rng = seeded_rng(1);
        let mut flow = start_flow(&oauth_settings(&server), CLIENT_ID, USER, &mut rng)
            .await
            .unwrap();
        assert_eq!(flow.phase(), FlowPhase::AwaitingRedirect);
        assert!(flow.redirect_uri().starts_with("http://127.0.0.1:"));
        let url = flow.authorize_url().to_string();
        let (approved, code) = tokio::join!(
            server.approve(&url, USER),
            wait_for_redirect(&mut flow, Duration::from_secs(5))
        );
        approved.unwrap();
        assert!(code.unwrap().expose().starts_with("code-"));
        assert_eq!(flow.phase(), FlowPhase::CodeReceived);
    })
    .await
    .unwrap();
}

/// L'URL d'autorisation est encodée et le défi correspond au vérificateur (S256)
#[tokio::test]
async fn authorize_url_is_encoded_and_challenge_matches_the_verifier() {
    timeout(LIMIT, async {
        let (server, _) = started_oauth().await;
        let mut rng = seeded_rng(2);
        let flow = start_flow(&oauth_settings(&server), CLIENT_ID, USER, &mut rng)
            .await
            .unwrap();
        let url = flow.authorize_url();
        assert!(url.starts_with(&server.authorize_url()));
        assert!(!url.contains(' ') && !url.contains('@'), "{url}");
        assert!(url.contains("login_hint=alice%40example.org"), "{url}");
        assert!(url.contains("scope=mail%20imap%20smtp"), "{url}");
        assert_eq!(query_value(url, "response_type").unwrap(), "code");
        assert_eq!(query_value(url, "client_id").unwrap(), CLIENT_ID);
        assert_eq!(query_value(url, "code_challenge_method").unwrap(), "S256");
        assert_eq!(
            query_value(url, "redirect_uri").unwrap(),
            flow.redirect_uri()
        );
        let challenge = challenge_for(flow.verifier().expose());
        assert_eq!(query_value(url, "code_challenge").unwrap(), challenge);
        assert!(query_value(url, "state").unwrap().len() >= 32);
    })
    .await
    .unwrap();
}

/// Un `state` modifié est refusé, le flux échoue et l'écouteur est fermé
#[tokio::test]
async fn tampered_state_is_refused_and_the_listener_closes() {
    timeout(LIMIT, async {
        let (server, _) = started_oauth().await;
        let mut rng = seeded_rng(3);
        let mut flow = start_flow(&oauth_settings(&server), CLIENT_ID, USER, &mut rng)
            .await
            .unwrap();
        let redirect = flow.redirect_uri().to_string();
        let (reply, outcome) = tokio::join!(
            get(&redirect, "/callback?code=abc&state=falsifie"),
            wait_for_redirect(&mut flow, Duration::from_secs(5))
        );
        assert_eq!(
            outcome.unwrap_err().code,
            "authentication.oauth_state_mismatch"
        );
        assert!(reply.starts_with("HTTP/1.1 200"), "{reply}");
        assert_eq!(flow.phase(), FlowPhase::Failed);
        let authority = redirect
            .strip_prefix("http://")
            .unwrap()
            .split('/')
            .next()
            .unwrap();
        assert!(tokio::net::TcpStream::connect(authority).await.is_err());
    })
    .await
    .unwrap();
}

/// Le refus de l'utilisateur annule le flux (authentication.oauth_cancelled)
#[tokio::test]
async fn user_denial_cancels_the_flow() {
    timeout(LIMIT, async {
        let (server, _) = started_oauth().await;
        let mut rng = seeded_rng(4);
        let mut flow = start_flow(&oauth_settings(&server), CLIENT_ID, USER, &mut rng)
            .await
            .unwrap();
        let url = flow.authorize_url().to_string();
        let (denied, outcome) = tokio::join!(
            server.deny(&url),
            wait_for_redirect(&mut flow, Duration::from_secs(5))
        );
        denied.unwrap();
        assert_eq!(outcome.unwrap_err().code, "authentication.oauth_cancelled");
        assert_eq!(flow.phase(), FlowPhase::Cancelled);
    })
    .await
    .unwrap();
}

/// Sans redirection, le délai expire (network.timeout) et le flux est en échec
#[tokio::test]
async fn missing_redirect_times_out() {
    timeout(LIMIT, async {
        let (server, _) = started_oauth().await;
        let mut rng = seeded_rng(5);
        let mut flow = start_flow(&oauth_settings(&server), CLIENT_ID, USER, &mut rng)
            .await
            .unwrap();
        let error = wait_for_redirect(&mut flow, Duration::from_millis(200))
            .await
            .unwrap_err();
        assert_eq!(error.code, "network.timeout");
        assert_eq!(flow.phase(), FlowPhase::Failed);
        let again = wait_for_redirect(&mut flow, Duration::from_millis(50)).await;
        assert_eq!(again.unwrap_err().code, "invalid_input.parameter");
    })
    .await
    .unwrap();
}
