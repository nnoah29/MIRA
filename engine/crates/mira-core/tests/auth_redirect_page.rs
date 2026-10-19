#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Pages affichées dans le navigateur à la fin de l'autorisation : succès, refus, et aucune valeur reflétée.

mod support_auth;

use std::time::Duration;

use mira_core::auth::{start_flow, wait_for_redirect};
use support_auth::browser::{get, query_value};
use support_auth::{CLIENT_ID, LIMIT, USER, oauth_settings, seeded_rng, started_oauth};
use tokio::time::timeout;

/// Joue la redirection `target` (sans `state`, ajouté ici) et rend la réponse HTTP du navigateur
async fn browser_reply(seed: u64, target: &str) -> String {
    let (server, _) = started_oauth().await;
    let mut rng = seeded_rng(seed);
    let mut flow = start_flow(&oauth_settings(&server), CLIENT_ID, USER, &mut rng)
        .await
        .unwrap();
    let redirect = flow.redirect_uri().to_string();
    let state = query_value(flow.authorize_url(), "state").unwrap();
    let target = format!("{target}&state={state}");
    let (_, reply) = tokio::join!(
        wait_for_redirect(&mut flow, Duration::from_secs(5)),
        get(&redirect, &target)
    );
    reply
}

/// La page de succès est habillée aux couleurs de MIRA, bilingue, sans script ni ressource externe
#[tokio::test]
async fn success_page_is_styled_bilingual_and_loads_nothing_external() {
    timeout(LIMIT, async {
        let reply = browser_reply(21, "/callback?code=secret-code-value").await;
        assert!(reply.starts_with("HTTP/1.1 200"), "{reply}");
        assert!(reply.contains("Autorisation reçue"), "{reply}");
        assert!(reply.contains("revenir à MIRA"), "{reply}");
        assert!(reply.contains("go back to MIRA"), "{reply}");
        assert!(reply.contains("#e8542b"), "{reply}");
        assert!(reply.contains("@font-face"), "{reply}");
        assert!(!reply.contains("<script"), "{reply}");
        assert!(
            !reply.contains("http://") && !reply.contains("https://"),
            "{reply}"
        );
        let policy = reply
            .lines()
            .find(|line| line.starts_with("Content-Security-Policy:"))
            .unwrap();
        assert!(policy.contains("default-src 'none'"), "{policy}");
        assert!(!policy.contains("script-src"), "{policy}");
        assert!(policy.contains("style-src 'sha256-"), "{policy}");
        assert!(!policy.contains("unsafe"), "{policy}");
    })
    .await
    .unwrap();
}

/// Un refus de l'utilisateur n'affiche plus « Autorisation reçue » mais une page d'échec
#[tokio::test]
async fn denial_shows_a_failure_page_instead_of_a_success_message() {
    timeout(LIMIT, async {
        let reply = browser_reply(22, "/callback?error=access_denied").await;
        assert!(reply.starts_with("HTTP/1.1 200"), "{reply}");
        assert!(reply.contains("Autorisation non aboutie"), "{reply}");
        assert!(reply.contains("réessayer"), "{reply}");
        assert!(!reply.contains("Autorisation reçue"), "{reply}");
        assert!(!reply.contains("<script"), "{reply}");
    })
    .await
    .unwrap();
}

/// Aucune valeur de la requête (code, message d'erreur) n'est reflétée dans la page
#[tokio::test]
async fn page_never_reflects_values_from_the_query() {
    timeout(LIMIT, async {
        let ok = browser_reply(23, "/callback?code=CODE-MARKER-123").await;
        assert!(!ok.contains("CODE-MARKER-123"), "{ok}");
        let failed = browser_reply(
            24,
            "/callback?error=server_error&error_description=%3Cb%3EMARKER-456%3C/b%3E",
        )
        .await;
        assert!(!failed.contains("MARKER-456"), "{failed}");
        assert!(failed.contains("Autorisation non aboutie"), "{failed}");
    })
    .await
    .unwrap();
}
