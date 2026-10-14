#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Écouteur de redirection : une connexion muette ne bloque pas les autres (audit 12).

mod support_auth;

use std::time::Duration;

use mira_core::auth::{start_flow, wait_for_redirect};
use support_auth::browser::{get, query_value};
use support_auth::{CLIENT_ID, LIMIT, USER, oauth_settings, seeded_rng, started_oauth};
use tokio::net::TcpStream;
use tokio::time::timeout;

/// Une connexion ouverte sans rien envoyer n'empêche pas la vraie redirection d'aboutir vite
#[tokio::test]
async fn regression_silent_connection_does_not_block_the_genuine_callback() {
    timeout(LIMIT, async {
        let (server, _) = started_oauth().await;
        let mut rng = seeded_rng(31);
        let mut flow = start_flow(&oauth_settings(&server), CLIENT_ID, USER, &mut rng)
            .await
            .unwrap();
        let redirect = flow.redirect_uri().to_string();
        let state = query_value(flow.authorize_url(), "state").unwrap();
        let authority = redirect.strip_prefix("http://").unwrap();
        let authority = authority.split('/').next().unwrap();
        let _silent = TcpStream::connect(authority).await.unwrap();
        let target = format!("/callback?code=ok&state={state}");
        let browser = get(&redirect, &target);
        let (outcome, reply) = tokio::join!(
            wait_for_redirect(&mut flow, Duration::from_millis(1500)),
            browser
        );
        assert_eq!(outcome.unwrap().expose(), "ok");
        assert!(reply.starts_with("HTTP/1.1 200"), "{reply}");
    })
    .await
    .unwrap();
}
