#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Écouteur de redirection : requêtes parasites, page de réponse et règle AG-03 sur les URL.

mod support_auth;

use std::time::Duration;

use mira_core::auth::form::{build_query, parse_query, percent_decode, percent_encode};
use mira_core::auth::loopback::{is_loopback_host, require_secure_url};
use mira_core::auth::{start_flow, wait_for_redirect};
use support_auth::browser::{get, query_value};
use support_auth::{CLIENT_ID, LIMIT, USER, oauth_settings, seeded_rng, started_oauth};
use tokio::time::timeout;

/// Une requête parasite reçoit 404 sans interrompre l'attente ; la page finale n'a pas de script
#[tokio::test]
async fn stray_requests_get_404_and_the_final_page_has_no_script() {
    timeout(LIMIT, async {
        let (server, _) = started_oauth().await;
        let mut rng = seeded_rng(7);
        let mut flow = start_flow(&oauth_settings(&server), CLIENT_ID, USER, &mut rng)
            .await
            .unwrap();
        let redirect = flow.redirect_uri().to_string();
        let state = query_value(flow.authorize_url(), "state").unwrap();
        let browser = async {
            let stray = get(&redirect, "/favicon.ico").await;
            let done = get(&redirect, &format!("/callback?code=xyz&state={state}")).await;
            (stray, done)
        };
        let (outcome, (stray, done)) = tokio::join!(
            wait_for_redirect(&mut flow, Duration::from_secs(5)),
            browser
        );
        assert_eq!(outcome.unwrap().expose(), "xyz");
        assert!(stray.starts_with("HTTP/1.1 404"), "{stray}");
        assert!(done.starts_with("HTTP/1.1 200"), "{done}");
        assert!(
            done.contains("fermer cette fenêtre") && !done.contains("<script"),
            "{done}"
        );
        assert!(done.contains("Referrer-Policy: no-referrer"), "{done}");
    })
    .await
    .unwrap();
}

/// L'écouteur n'accepte que des connexions de la boucle locale
#[tokio::test]
async fn listener_is_bound_to_the_loopback_interface_only() {
    timeout(LIMIT, async {
        let (server, _) = started_oauth().await;
        let mut rng = seeded_rng(8);
        let flow = start_flow(&oauth_settings(&server), CLIENT_ID, USER, &mut rng)
            .await
            .unwrap();
        let authority = flow.redirect_uri().strip_prefix("http://").unwrap();
        assert!(authority.starts_with("127.0.0.1:"), "{authority}");
        assert!(flow.redirect_uri().ends_with("/callback"));
    })
    .await
    .unwrap();
}

/// Un point d'autorisation ou de jetons en HTTP hors boucle locale est refusé au démarrage
#[tokio::test]
async fn start_refuses_plaintext_endpoints_outside_the_loopback() {
    timeout(LIMIT, async {
        let (server, _) = started_oauth().await;
        let mut rng = seeded_rng(9);
        let mut settings = oauth_settings(&server);
        settings.token_url = "http://auth.example.org/token".to_string();
        let error = start_flow(&settings, CLIENT_ID, USER, &mut rng)
            .await
            .unwrap_err();
        assert_eq!(error.code, "tls.plaintext_refused");
        let mut settings = oauth_settings(&server);
        settings.authorize_url = "http://auth.example.org/authorize".to_string();
        let error = start_flow(&settings, CLIENT_ID, USER, &mut rng)
            .await
            .unwrap_err();
        assert_eq!(error.code, "tls.plaintext_refused");
        settings.authorize_url = "https://auth.example.org/authorize".to_string();
        assert!(
            start_flow(&settings, CLIENT_ID, USER, &mut rng)
                .await
                .is_ok()
        );
    })
    .await
    .unwrap();
}

/// La règle AG-03 sur les URL : HTTPS partout, HTTP seulement vers la boucle locale
#[test]
fn plaintext_urls_are_accepted_only_for_the_loopback() {
    for good in [
        "https://auth.example.org/token",
        "http://127.0.0.1:8080/token",
        "http://localhost/token",
        "http://[::1]:9/token",
    ] {
        assert!(require_secure_url(good).is_ok(), "{good}");
    }
    for bad in [
        "http://auth.example.org/token",
        "http://127.0.0.1.example.org/token",
        "http://127.0.0.1@auth.example.org/token",
        "ftp://127.0.0.1/token",
        "127.0.0.1:80/token",
    ] {
        assert_eq!(
            require_secure_url(bad).unwrap_err().code,
            "tls.plaintext_refused",
            "{bad}"
        );
    }
    assert!(is_loopback_host("LOCALHOST") && !is_loopback_host("10.0.0.1"));
}

/// L'encodage de formulaire protège les caractères réservés et se décode sans perte
#[test]
fn form_encoding_round_trips_reserved_characters() {
    let value = "a b&c=d/é@+%";
    let encoded = percent_encode(value);
    assert!(
        !encoded.contains(|c: char| " &=/é@+".contains(c)),
        "{encoded}"
    );
    assert_eq!(percent_decode(&encoded), value);
    assert_eq!(percent_decode("A+b%zz%4"), "A b%zz%4");
    let query = build_query(&[("k", value), ("x", "1")]);
    let pairs = parse_query(&query);
    assert_eq!(
        pairs,
        vec![("k".into(), value.into()), ("x".into(), "1".into())]
    );
}
