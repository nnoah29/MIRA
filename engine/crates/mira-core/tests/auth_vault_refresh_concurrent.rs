#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Rafraîchissements simultanés d'un même compte : un seul aller-retour réseau (revue 14).

mod support_auth;

use support_auth::flow::{approved_flow, clock};
use support_auth::vault::{fixture, ids};
use support_auth::{CLIENT_ID, LIMIT, USER, started_oauth};
use tokio::time::timeout;

/// Cinq appelants qui ont vu le même jeton périmé obtiennent tous le même jeton neuf
#[tokio::test]
async fn regression_concurrent_refreshes_issue_a_single_token() {
    timeout(LIMIT, async {
        let (server, credentials) = started_oauth().await;
        let (flow, tokens) = approved_flow(&server, 51).await;
        let fixture = fixture();
        let (profile, account) = ids();
        fixture
            .vault
            .store_tokens(&profile, &account, &tokens)
            .unwrap();
        server.expire_access_tokens(USER);
        let (url, clock) = (flow.token_url(), clock());
        let stale = Some(&tokens.access_token);
        let refresh = || {
            fixture
                .vault
                .refresh_access_token_if_stale(&profile, &account, url, CLIENT_ID, &clock, stale)
        };
        let results = tokio::join!(refresh(), refresh(), refresh(), refresh(), refresh());
        let first = results.0.unwrap();
        for other in [results.1, results.2, results.3, results.4] {
            assert_eq!(other.unwrap().expose(), first.expose());
        }
        assert!(credentials.check_access_token(USER, first.expose()));
        assert_ne!(first.expose(), tokens.access_token.expose());
    })
    .await
    .unwrap();
}

/// Un jeton déjà renouvelé par un autre appelant n'est pas renouvelé une seconde fois
#[tokio::test]
async fn already_renewed_token_is_returned_without_a_new_refresh() {
    timeout(LIMIT, async {
        let (server, _) = started_oauth().await;
        let (flow, tokens) = approved_flow(&server, 52).await;
        let fixture = fixture();
        let (profile, account) = ids();
        let vault = &fixture.vault;
        vault.store_tokens(&profile, &account, &tokens).unwrap();
        let (url, clock) = (flow.token_url(), clock());
        let renewed = vault
            .refresh_access_token(&profile, &account, url, CLIENT_ID, &clock)
            .await
            .unwrap();
        let stale = Some(&tokens.access_token);
        let again = vault
            .refresh_access_token_if_stale(&profile, &account, url, CLIENT_ID, &clock, stale)
            .await
            .unwrap();
        assert_eq!(again.expose(), renewed.expose());
    })
    .await
    .unwrap();
}
