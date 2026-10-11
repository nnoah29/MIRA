#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Ajout de compte OAuth2 par la façade : flux PKCE contre le faux serveur, IMAP en XOAUTH2 (S1, 15.2).

mod support_api;

use mira_core::api::{AccountRequest, CredentialInput, Request, Response};
use mira_vocabulary::{AccountState, AuthMethodKind, EventKind, SecretKind};
use support_api::fixture;
use support_api::oauth::{OAUTH_USER, authorize};
use support_api::oauth_requests::{add_oauth_request, oauth_draft};
use support_api::queries::{accounts, event_kinds, identities};

/// S1 : un compte OAuth2 est ajouté, connecté en XOAUTH2, avec ses jetons au trousseau
#[tokio::test]
async fn scenario_s1_add_oauth_account() {
    let f = fixture::start().await;
    let flow = authorize(&f, OAUTH_USER).await;
    let response = f
        .engine
        .execute(add_oauth_request(&flow, "key-s1"))
        .await
        .unwrap();
    let Response::Account(created) = response else {
        panic!("compte attendu");
    };
    assert_eq!(created.auth, AuthMethodKind::OAuth2);
    assert_eq!(created.state, AccountState::Connected);
    assert_eq!(created.provider.as_str(), "fake_oauth");
    assert!(
        f.stored_secret(&created.id, SecretKind::AccessToken)
            .is_some()
    );
    assert!(
        f.stored_secret(&created.id, SecretKind::RefreshToken)
            .is_some()
    );
    assert!(f.stored_secret(&created.id, SecretKind::Password).is_none());
    assert_eq!(identities(&f.engine, &created.id).await.len(), 1);
    let kinds = event_kinds(&f.engine).await;
    assert!(kinds.contains(&EventKind::AccountAdded {
        account: created.id
    }));
}

/// Le flux est consommé par l'ajout : le rejouer avec une autre clé est refusé
#[tokio::test]
async fn oauth_flow_cannot_create_two_accounts() {
    let f = fixture::start().await;
    let flow = authorize(&f, OAUTH_USER).await;
    f.engine
        .execute(add_oauth_request(&flow, "first"))
        .await
        .unwrap();
    let error = f
        .engine
        .execute(add_oauth_request(&flow, "second"))
        .await
        .unwrap_err();
    assert_eq!(error.code, "invalid_input.parameter");
    assert_eq!(accounts(&f.engine).await.len(), 1);
}

/// Le test de connexion OAuth n'enregistre rien et laisse le flux utilisable pour l'ajout
#[tokio::test]
async fn test_connection_with_oauth_registers_nothing() {
    let f = fixture::start().await;
    let flow = authorize(&f, OAUTH_USER).await;
    let credential = CredentialInput::OAuthFlow { flow: flow.clone() };
    let request = Request::Account(AccountRequest::TestConnection {
        draft: oauth_draft(),
        credential,
    });
    assert_eq!(f.engine.execute(request).await.unwrap(), Response::Done);
    assert!(accounts(&f.engine).await.is_empty());
    f.engine
        .execute(add_oauth_request(&flow, "k"))
        .await
        .unwrap();
    assert_eq!(accounts(&f.engine).await.len(), 1);
}

/// Jeton refusé par le serveur : rien n'est enregistré, les jetons sont supprimés, le flux est clos
#[tokio::test]
async fn oauth_add_with_rejected_token_registers_nothing() {
    let f = fixture::start().await;
    let flow = authorize(&f, OAUTH_USER).await;
    assert_eq!(f.secrets.entry_count(), 2);
    f.credentials
        .revoke_access_token(&reserved_access_token(&f));
    let error = f
        .engine
        .execute(add_oauth_request(&flow, "k"))
        .await
        .unwrap_err();
    assert_eq!(error.code, "authentication.token_expired");
    assert!(accounts(&f.engine).await.is_empty());
    assert_eq!(f.secrets.entry_count(), 0);
    let retry = f
        .engine
        .execute(add_oauth_request(&flow, "k2"))
        .await
        .unwrap_err();
    assert_eq!(retry.code, "invalid_input.parameter");
}

/// Supprimer un compte OAuth2 efface ses jetons et laisse le serveur intact
#[tokio::test]
async fn deleting_oauth_account_removes_its_tokens() {
    let f = fixture::start().await;
    let flow = authorize(&f, OAUTH_USER).await;
    let Response::Account(created) = f
        .engine
        .execute(add_oauth_request(&flow, "k"))
        .await
        .unwrap()
    else {
        panic!("compte attendu");
    };
    let delete = Request::Account(AccountRequest::DeleteAccount {
        account: created.id.clone(),
    });
    f.engine.execute(delete).await.unwrap();
    assert_eq!(f.secrets.entry_count(), 0);
    assert!(
        f.stored_secret(&created.id, SecretKind::AccessToken)
            .is_none()
    );
}

/// Jeton d'accès du premier compte réservé (identifiant « id-000001 » avec les identifiants de test)
fn reserved_access_token(f: &fixture::Fixture) -> String {
    let account = mira_vocabulary::AccountId::from_raw("id-000001");
    f.stored_secret(&account, SecretKind::AccessToken).unwrap()
}
