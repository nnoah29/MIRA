#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Domaine « Ai » par la façade : adresses refusées (identifiants, requête, clair hors boucle locale).

mod ai_models_support;
mod support_ai_api;
mod support_api;

use support_ai_api::calls::{add, list};
use support_ai_api::operations::update;
use support_ai_api::providers::{EDITABLE, started};

const REFUSED: &str = "invalid_input.ai_url_invalid";

/// Des identifiants dans l'adresse sont refusés, à la création comme à la mise à jour
#[tokio::test]
async fn generic_url_with_credentials_is_refused() {
    let f = started().await;
    for url in [
        "https://user:pass@api.example.test/v1",
        "https://user@api.example.test/v1",
    ] {
        assert_eq!(
            add(&f.engine, EDITABLE, "A", Some(url)).await.unwrap_err(),
            REFUSED
        );
    }
    let created = add(
        &f.engine,
        EDITABLE,
        "A",
        Some("https://api.example.test/v1"),
    )
    .await
    .unwrap();
    let url = "https://user:pass@api.example.test/v1";
    let error = update(&f.engine, &created.id, "A", Some(url), None)
        .await
        .unwrap_err();
    assert_eq!(error, REFUSED);
    assert_eq!(
        list(&f.engine).await[0].base_url.as_deref(),
        Some("https://api.example.test/v1")
    );
}

/// Une requête ou un fragment dans l'adresse sont refusés
#[tokio::test]
async fn generic_url_with_query_or_fragment_is_refused() {
    let f = started().await;
    for url in [
        "https://api.example.test/v1?key=abc",
        "https://api.example.test/v1?",
        "https://api.example.test/v1#frag",
        "https://api.example.test/v1#",
    ] {
        assert_eq!(
            add(&f.engine, EDITABLE, "A", Some(url)).await.unwrap_err(),
            REFUSED,
            "{url}"
        );
    }
    assert!(list(&f.engine).await.is_empty());
}

/// Les `/` de fin sont retirés, les blancs de bord aussi
#[tokio::test]
async fn trailing_slash_is_removed() {
    let f = started().await;
    let created = add(
        &f.engine,
        EDITABLE,
        "A",
        Some("  https://api.example.test/v1/ "),
    )
    .await
    .unwrap();
    assert_eq!(
        created.base_url.as_deref(),
        Some("https://api.example.test/v1")
    );
    assert_eq!(list(&f.engine).await[0].base_url, created.base_url);
}

/// Le clair est refusé hors boucle locale, comme tout schéma autre que https
#[tokio::test]
async fn http_is_refused_outside_loopback() {
    let f = started().await;
    for url in [
        "http://api.example.test/v1",
        "ftp://api.example.test/v1",
        "pas une adresse",
        "https://",
        "http://127.0.0.1.example.test/v1",
    ] {
        assert_eq!(
            add(&f.engine, EDITABLE, "A", Some(url)).await.unwrap_err(),
            REFUSED,
            "{url}"
        );
    }
    assert!(list(&f.engine).await.is_empty());
}
