#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Flux OAuth2 : vérificateur aléatoire conforme et secrets absents de l'affichage.

mod support_auth;

use std::time::Duration;

use mira_core::auth::{start_flow, wait_for_redirect};
use support_auth::browser::query_value;
use support_auth::{CLIENT_ID, LIMIT, USER, oauth_settings, seeded_rng, started_oauth};
use tokio::time::timeout;

/// Le vérificateur fait 64 caractères non réservés et change d'un flux à l'autre
#[tokio::test]
async fn verifier_is_random_and_made_of_unreserved_characters() {
    timeout(LIMIT, async {
        let (server, _) = started_oauth().await;
        let mut rng = seeded_rng(10);
        let settings = oauth_settings(&server);
        let first = start_flow(&settings, CLIENT_ID, USER, &mut rng)
            .await
            .unwrap();
        let second = start_flow(&settings, CLIENT_ID, USER, &mut rng)
            .await
            .unwrap();
        let verifier = first.verifier().expose();
        assert_eq!(verifier.len(), 64);
        assert!(
            verifier
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-._~".contains(&b))
        );
        assert_ne!(verifier, second.verifier().expose());
        assert_ne!(first.authorize_url(), second.authorize_url());
        assert_ne!(first.redirect_uri(), second.redirect_uri());
    })
    .await
    .unwrap();
}

/// Le `Debug` d'un flux et d'un code ne montre ni vérificateur, ni état, ni code
#[tokio::test]
async fn debug_output_hides_verifier_state_and_code() {
    timeout(LIMIT, async {
        let (server, _) = started_oauth().await;
        let mut rng = seeded_rng(6);
        let mut flow = start_flow(&oauth_settings(&server), CLIENT_ID, USER, &mut rng)
            .await
            .unwrap();
        let url = flow.authorize_url().to_string();
        let verifier = flow.verifier().expose().to_string();
        let state = query_value(&url, "state").unwrap();
        let (_, code) = tokio::join!(
            server.approve(&url, USER),
            wait_for_redirect(&mut flow, Duration::from_secs(5))
        );
        let code = code.unwrap();
        let shown = format!("{flow:?} {code:?} {:?}", flow.verifier());
        for secret in [verifier.as_str(), state.as_str(), code.expose()] {
            assert!(!shown.contains(secret), "{shown}");
        }
    })
    .await
    .unwrap();
}
