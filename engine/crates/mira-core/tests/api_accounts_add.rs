#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Ajout de compte par la façade : mot de passe d'application, serveurs saisis, identité par défaut (S2, S3, 7.5).

mod support_api;

use mira_core::api::{AccountRequest, CredentialInput, Request, Response, SecretInput};
use mira_vocabulary::{AccountState, AuthMethodKind, EventKind, SecretKind};
use support_api::endpoints;
use support_api::fixture::{self, LABELS_PASSWORD, LABELS_USER};
use support_api::queries::{accounts, event_kinds, identities};
use support_api::requests::{add_account, draft, manual_draft, prepare};

/// S2 : un compte à mot de passe d'application est ajouté, connecté, et son secret est au trousseau
#[tokio::test]
async fn scenario_s2_add_app_password_account() {
    let f = fixture::start().await;
    let preparation = prepare(&f.engine, LABELS_USER).await;
    assert_eq!(preparation.provider.id.as_str(), "fake_labels");
    assert_eq!(preparation.servers.imap.host, "127.0.0.1");
    let created = add_account(
        &f.engine,
        draft(LABELS_USER, AuthMethodKind::AppPassword),
        LABELS_PASSWORD,
        "key-s2",
    )
    .await;
    assert_eq!(created.state, AccountState::Connected);
    assert_eq!(created.auth, AuthMethodKind::AppPassword);
    assert_eq!(created.provider.as_str(), "fake_labels");
    assert_eq!(accounts(&f.engine).await, vec![created.clone()]);
    let stored = f.stored_secret(&created.id, SecretKind::Password);
    assert_eq!(stored.as_deref(), Some(LABELS_PASSWORD));
    let kinds = event_kinds(&f.engine).await;
    assert!(kinds.contains(&EventKind::AccountAdded {
        account: created.id
    }));
}

/// S3 : un compte « Autre » avec serveurs saisis est testé sans rien enregistrer, puis ajouté
#[tokio::test]
async fn scenario_s3_add_other_account_with_manual_servers() {
    let f = fixture::start().await;
    f.credentials.add_password("carol@other.test", "other-pass");
    let servers = endpoints::local_servers(f.imap.port(), f.smtp.port());
    let other = manual_draft("carol@other.test", servers.clone(), "fake_fallback");
    let tested = f
        .engine
        .execute(Request::Account(AccountRequest::TestConnection {
            draft: other.clone(),
            credential: CredentialInput::Password(SecretInput::new("other-pass")),
        }))
        .await
        .unwrap();
    assert_eq!(tested, Response::Done);
    assert!(accounts(&f.engine).await.is_empty());
    assert_eq!(f.secrets.entry_count(), 0);
    let created = add_account(&f.engine, other, "other-pass", "key-s3").await;
    assert_eq!(created.servers, Some(servers));
    assert_eq!(created.provider.as_str(), "fake_fallback");
    assert_eq!(f.secrets.entry_count(), 1);
}

/// Invariant 11 : tout compte ajouté a exactement une identité, celle par défaut, à son adresse
#[tokio::test]
async fn invariant_11_added_account_has_default_identity() {
    let f = fixture::start().await;
    let created = add_account(
        &f.engine,
        draft(fixture::FOLDERS_USER, AuthMethodKind::Password),
        fixture::FOLDERS_PASSWORD,
        "key-11",
    )
    .await;
    let list = identities(&f.engine, &created.id).await;
    assert_eq!(list.len(), 1);
    assert!(list[0].is_default);
    assert_eq!(list[0].address, created.address);
    assert_eq!(list[0].display_name, created.display_name);
}

/// Un mot de passe refusé : rien n'est enregistré, le secret est supprimé, aucun événement d'ajout
#[tokio::test]
async fn failed_connection_test_registers_nothing() {
    let f = fixture::start().await;
    let request = support_api::requests::add_request(
        draft(fixture::FOLDERS_USER, AuthMethodKind::Password),
        "mauvais",
        "key-bad",
    );
    let error = f.engine.execute(request).await.unwrap_err();
    assert_eq!(error.code, "authentication.rejected");
    assert!(accounts(&f.engine).await.is_empty());
    assert_eq!(f.secrets.entry_count(), 0);
    let kinds = event_kinds(&f.engine).await;
    assert!(
        !kinds
            .iter()
            .any(|kind| matches!(kind, EventKind::AccountAdded { .. }))
    );
}

/// La même clé d'idempotence redonne le même compte ; une autre adresse avec cette clé est un conflit
#[tokio::test]
async fn add_account_is_idempotent() {
    let f = fixture::start().await;
    let first = add_account(
        &f.engine,
        draft(LABELS_USER, AuthMethodKind::AppPassword),
        LABELS_PASSWORD,
        "same",
    )
    .await;
    let again = add_account(
        &f.engine,
        draft(LABELS_USER, AuthMethodKind::AppPassword),
        LABELS_PASSWORD,
        "same",
    )
    .await;
    assert_eq!(first, again);
    assert_eq!(accounts(&f.engine).await.len(), 1);
    assert_eq!(identities(&f.engine, &first.id).await.len(), 1);
    let other = support_api::requests::add_request(
        draft(fixture::FOLDERS_USER, AuthMethodKind::Password),
        fixture::FOLDERS_PASSWORD,
        "same",
    );
    let error = f.engine.execute(other).await.unwrap_err();
    assert_eq!(error.code, "invalid_input.idempotency_conflict");
}

/// Une méthode que le provider ne propose pas, ou un secret d'un autre type, sont refusés avant tout réseau
#[tokio::test]
async fn add_account_refuses_unsupported_or_mismatched_authentication() {
    let f = fixture::start().await;
    let unsupported = draft(LABELS_USER, AuthMethodKind::Password);
    let request = support_api::requests::add_request(unsupported, LABELS_PASSWORD, "k1");
    let error = f.engine.execute(request).await.unwrap_err();
    assert_eq!(error.code, "invalid_input.parameter");
    let mismatch = Request::Account(AccountRequest::TestConnection {
        draft: draft(LABELS_USER, AuthMethodKind::AppPassword),
        credential: CredentialInput::Password(SecretInput::new(LABELS_PASSWORD)),
    });
    let error = f.engine.execute(mismatch).await.unwrap_err();
    assert_eq!(error.code, "invalid_input.parameter");
    assert_eq!(f.imap.connection_count(), 0);
}
