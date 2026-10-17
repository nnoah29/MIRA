#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Mot de passe d'application par la façade : remplacement dans le trousseau (7.7, S2).

mod support_api;

use mira_core::api::{AuthRequest, Request, Response, SecretInput};
use mira_vocabulary::{AccountId, AuthMethodKind, SecretKind};
use support_api::fixture::{self, LABELS_PASSWORD, LABELS_USER};
use support_api::requests::{add_account, draft};

/// Un nouveau mot de passe d'application remplace l'ancien dans le trousseau et n'est jamais renvoyé
#[tokio::test]
async fn store_app_password_replaces_the_stored_secret() {
    let f = fixture::start().await;
    let account = add_account(
        &f.engine,
        draft(LABELS_USER, AuthMethodKind::AppPassword),
        LABELS_PASSWORD,
        "k",
    )
    .await;
    let request = Request::Auth(AuthRequest::StoreAppPassword {
        account: account.id.clone(),
        password: SecretInput::new("nouveau-secret-42"),
    });
    let response = f.engine.execute(request).await.unwrap();
    assert!(matches!(response, Response::Account(ref a) if a.id == account.id));
    let stored = f.stored_secret(&account.id, SecretKind::Password);
    assert_eq!(stored.as_deref(), Some("nouveau-secret-42"));
    assert!(
        !serde_json::to_string(&response)
            .unwrap()
            .contains("nouveau-secret-42")
    );
}

/// Un mot de passe vide ou un compte inconnu sont refusés avec un code stable
#[tokio::test]
async fn store_app_password_refuses_empty_or_unknown_account() {
    let f = fixture::start().await;
    let account = add_account(
        &f.engine,
        draft(LABELS_USER, AuthMethodKind::AppPassword),
        LABELS_PASSWORD,
        "k",
    )
    .await;
    let empty = Request::Auth(AuthRequest::StoreAppPassword {
        account: account.id.clone(),
        password: SecretInput::new(""),
    });
    assert_eq!(
        f.engine.execute(empty).await.unwrap_err().code,
        "invalid_input.parameter"
    );
    let ghost = Request::Auth(AuthRequest::StoreAppPassword {
        account: AccountId::from_raw("ghost"),
        password: SecretInput::new("x"),
    });
    assert_eq!(
        f.engine.execute(ghost).await.unwrap_err().code,
        "not_found.account"
    );
    let stored = f.stored_secret(&account.id, SecretKind::Password);
    assert_eq!(stored.as_deref(), Some(LABELS_PASSWORD));
}
