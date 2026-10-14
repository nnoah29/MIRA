#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Client OAuth qui présente un secret d'application : échange du code et rafraîchissement.

mod support_auth;

use std::time::Duration;

use mira_core::auth::{
    complete_flow, refresh_tokens, refresh_tokens_with_secret, start_flow, wait_for_redirect,
};
use mira_core::ports::SecretValue;
use support_auth::flow::clock;
use support_auth::{CLIENT_ID, USER, oauth_settings, seeded_rng, started_oauth};

const SECRET: &str = "s3cr3t-value";

/// Le flux présente le secret fixé : l'échange réussit, puis le rafraîchissement exige le secret
#[tokio::test]
async fn secret_is_presented_at_exchange_and_refresh() {
    let (server, _) = started_oauth().await;
    server.require_client_secret(SECRET);
    let mut rng = seeded_rng(31);
    let mut flow = start_flow(&oauth_settings(&server), CLIENT_ID, USER, &mut rng)
        .await
        .unwrap();
    flow.set_client_secret(Some(SecretValue::new(SECRET)));
    let url = flow.authorize_url().to_string();
    let (approved, code) = tokio::join!(
        server.approve(&url, USER),
        wait_for_redirect(&mut flow, Duration::from_secs(5))
    );
    approved.unwrap();
    let tokens = complete_flow(&mut flow, &code.unwrap(), &clock())
        .await
        .unwrap();
    let refresh = tokens.refresh_token.unwrap();
    let secret = SecretValue::new(SECRET);
    let client = (CLIENT_ID, Some(&secret));
    let fresh = refresh_tokens_with_secret(flow.token_url(), client, &refresh, &clock()).await;
    assert!(fresh.is_ok());
    let without = refresh_tokens(flow.token_url(), CLIENT_ID, &refresh, &clock()).await;
    assert!(without.is_err());
}

/// Sans secret fixé sur le flux, l'échange du code est refusé par un service qui en exige un
#[tokio::test]
async fn exchange_without_secret_is_refused_when_required() {
    let (server, _) = started_oauth().await;
    server.require_client_secret(SECRET);
    let mut rng = seeded_rng(32);
    let mut flow = start_flow(&oauth_settings(&server), CLIENT_ID, USER, &mut rng)
        .await
        .unwrap();
    let url = flow.authorize_url().to_string();
    let (approved, code) = tokio::join!(
        server.approve(&url, USER),
        wait_for_redirect(&mut flow, Duration::from_secs(5))
    );
    approved.unwrap();
    let result = complete_flow(&mut flow, &code.unwrap(), &clock()).await;
    assert!(result.is_err());
}
