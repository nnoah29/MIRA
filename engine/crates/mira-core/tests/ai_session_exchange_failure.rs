#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Session d'abonné : un échange refusé dit pourquoi (erreur OAuth du service) sans jamais recopier un secret.

mod session_support;

use mira_core::auth::complete_session_flow;
use session_support::browser::query_of;
use session_support::connect::NOW;
use session_support::flow::{HOST, login, receive, start};
use session_support::token_server::TokenServer;

/// Joue la connexion jusqu'à l'échange, auquel le service répond `status` et `body`
async fn refused(status: u16, body: &str) -> mira_vocabulary::MiraError {
    let server = TokenServer::start().await;
    server.push(status, body);
    let login = login(&server.url);
    let mut flow = start(&login, None).await;
    let query = query_of(flow.authorize_url());
    let extra = "code=CODE-SECRET-7&client_id=oaiapp_new";
    let code = receive(&mut flow, &query["state"], extra).await.unwrap();
    complete_session_flow(&mut flow, &login, &code, None, HOST, NOW)
        .await
        .unwrap_err()
}

/// Le message nomme l'erreur OAuth et sa description, et le code d'erreur n'est pas « accès révoqué »
#[tokio::test]
async fn refused_exchange_reports_the_oauth_error() {
    let body = r#"{"error":"invalid_grant","error_description":"redirect_uri mismatch"}"#;
    let error = refused(400, body).await;
    assert_eq!(error.code, "authentication.rejected");
    assert!(error.message.contains("400"), "{}", error.message);
    assert!(error.message.contains("invalid_grant"), "{}", error.message);
    assert!(
        error.message.contains("redirect_uri mismatch"),
        "{}",
        error.message
    );
}

/// Une description qui recopierait le code reçu est écartée ; un serveur en panne reste un échec passager
#[tokio::test]
async fn description_echoing_a_secret_is_dropped_and_5xx_stays_retryable() {
    let body = r#"{"error":"invalid_grant","error_description":"bad CODE-SECRET-7"}"#;
    let error = refused(400, body).await;
    assert!(error.message.contains("invalid_grant"));
    assert!(!error.message.contains("CODE-SECRET-7"));
    let down = refused(503, "{}").await;
    assert!(down.retryable);
}
