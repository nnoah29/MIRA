#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Domaine « Ai » par la façade : adresse obligatoire pour un fournisseur libre, interdite pour un fournisseur fixe.

mod ai_models_support;
mod support_ai_api;
mod support_api;

use support_ai_api::calls::{add, list};
use support_ai_api::providers::{EDITABLE, FIXED, started};

const REFUSED: &str = "invalid_input.ai_url_invalid";

/// Le clair vers la boucle locale est accepté
#[tokio::test]
async fn http_loopback_is_accepted() {
    let f = started().await;
    for url in [
        "http://127.0.0.1:1234/v1",
        "http://localhost:8080",
        "http://[::1]:9/v1",
    ] {
        let created = add(&f.engine, EDITABLE, "A", Some(url)).await.unwrap();
        assert_eq!(created.base_url.as_deref(), Some(url));
    }
    assert_eq!(list(&f.engine).await.len(), 3);
}

/// Un fournisseur à adresse fixe refuse une adresse propre, et n'en garde aucune
#[tokio::test]
async fn fixed_provider_refuses_a_custom_url() {
    let f = started().await;
    let error = add(&f.engine, FIXED, "A", Some("https://api.example.test/v1"))
        .await
        .unwrap_err();
    assert_eq!(error, REFUSED);
    let created = add(&f.engine, FIXED, "A", None).await.unwrap();
    assert_eq!(created.base_url, None);
}

/// Un fournisseur à adresse libre exige une adresse
#[tokio::test]
async fn generic_requires_a_url() {
    let f = started().await;
    assert_eq!(
        add(&f.engine, EDITABLE, "A", None).await.unwrap_err(),
        REFUSED
    );
    assert_eq!(
        add(&f.engine, EDITABLE, "A", Some("   "))
            .await
            .unwrap_err(),
        REFUSED
    );
    assert!(list(&f.engine).await.is_empty());
}

/// Un fournisseur inconnu, ou un libellé vide, est refusé
#[tokio::test]
async fn unknown_provider_or_empty_label_is_refused() {
    let f = started().await;
    let unknown = add(&f.engine, "inconnu", "A", None).await.unwrap_err();
    assert_eq!(unknown, "not_found.ai_provider");
    let empty = add(&f.engine, FIXED, "   ", None).await.unwrap_err();
    assert_eq!(empty, "invalid_input.parameter");
}
