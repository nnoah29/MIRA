#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Échange du code contre des jetons (15.2) : succès, refus, réponses anormales.

mod support_auth;

use mira_core::auth::{AuthorizationCode, FlowPhase, complete_flow, exchange_code, refresh_tokens};
use mira_core::ports::SecretValue;
use support_auth::canned::{exchange_failure, serve_once};
use support_auth::flow::{approved_flow, clock};
use support_auth::{LIMIT, USER, started_oauth};
use tokio::time::timeout;

/// L'échange donne des jetons acceptés par le service, avec expiration et sans fuite à l'affichage
#[tokio::test]
async fn exchange_yields_accepted_tokens_with_expiry() {
    timeout(LIMIT, async {
        let (server, credentials) = started_oauth().await;
        let (mut flow, tokens) = approved_flow(&server, 11).await;
        let access = tokens.access_token.expose().to_string();
        assert!(credentials.check_access_token(USER, &access));
        assert!(tokens.refresh_token.is_some());
        assert_eq!(
            tokens.expires_at.unwrap().as_millis(),
            1_000_000 + 3_600_000
        );
        assert_eq!(flow.phase(), FlowPhase::TokensObtained);
        let shown = format!("{tokens:?} {flow:?}");
        assert!(!shown.contains(&access), "{shown}");
        let refresh = tokens.refresh_token.as_ref().unwrap().expose();
        assert!(!shown.contains(refresh), "{shown}");
        let again = complete_flow(&mut flow, &AuthorizationCode::new("x"), &clock()).await;
        assert_eq!(again.unwrap_err().code, "invalid_input.parameter");
    })
    .await
    .unwrap();
}

/// Un mauvais vérificateur ou un code rejoué donne authentication.token_revoked
#[tokio::test]
async fn wrong_verifier_and_replayed_code_are_refused() {
    timeout(LIMIT, async {
        let (server, _) = started_oauth().await;
        let (flow, _) = approved_flow(&server, 12).await;
        let wrong = SecretValue::new("x".repeat(43));
        let code = AuthorizationCode::new("code-000001");
        let error = exchange_code(
            flow.token_url(),
            "mira-test-client",
            &code,
            &wrong,
            flow.redirect_uri(),
            &clock(),
        )
        .await
        .unwrap_err();
        assert_eq!(error.code, "authentication.token_revoked");
        let shown = format!("{error:?}");
        assert!(
            !shown.contains("code-000001") && !shown.contains(&"x".repeat(43)),
            "{shown}"
        );
        assert_eq!(
            error.suggested_action,
            Some(mira_vocabulary::SuggestedAction::Reauthenticate)
        );
    })
    .await
    .unwrap();
}

/// Un point d'accès de jetons en HTTP hors boucle locale est refusé sans connexion (AG-03)
#[tokio::test]
async fn plaintext_token_endpoint_outside_loopback_is_refused() {
    timeout(LIMIT, async {
        let code = AuthorizationCode::new("c");
        let verifier = SecretValue::new("v".repeat(43));
        let url = "http://auth.example.org/token";
        let error = exchange_code(
            url,
            "client",
            &code,
            &verifier,
            "http://127.0.0.1:1/callback",
            &clock(),
        )
        .await
        .unwrap_err();
        assert_eq!(error.code, "tls.plaintext_refused");
    })
    .await
    .unwrap();
}

/// Réponses anormales : JSON illisible, erreur 5xx réessayable, refus générique
#[tokio::test]
async fn abnormal_token_responses_map_to_contract_errors() {
    timeout(LIMIT, async {
        let garbled = exchange_failure(serve_once("200 OK", "pas du json").await).await;
        assert_eq!(garbled.code, "server.protocol");
        let down = exchange_failure(serve_once("503 Service Unavailable", "").await).await;
        assert_eq!(
            (down.code.as_str(), down.retryable),
            ("server.rejected", true)
        );
        let body = "{\"error\":\"invalid_client\"}";
        let refused = exchange_failure(serve_once("400 Bad Request", body).await).await;
        assert_eq!(refused.code, "authentication.rejected");
    })
    .await
    .unwrap();
}

/// Un point d'accès injoignable donne network.unreachable, réessayable
#[tokio::test]
async fn unreachable_token_endpoint_is_retryable() {
    timeout(LIMIT, async {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let url = format!("http://127.0.0.1:{port}/token");
        let refresh = SecretValue::new("r");
        let error = refresh_tokens(&url, "client", &refresh, &clock())
            .await
            .unwrap_err();
        assert_eq!(error.code, "network.unreachable");
        assert!(error.retryable);
    })
    .await
    .unwrap();
}
