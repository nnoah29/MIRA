#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Identités d'envoi par la façade : ajout, modification, identité par défaut unique (7.5).

mod support_api;

use mira_core::api::{IdentityRequest, NewIdentity, Request, Response};
use mira_vocabulary::{AccountId, Identity, IdentityId};
use support_api::fixture::{self, Fixture};
use support_api::queries::identities;

/// Exécute une requête d'identité et rend l'identité de la réponse
async fn identity_response(f: &Fixture, request: IdentityRequest) -> Result<Identity, String> {
    match f.engine.execute(Request::Identity(request)).await {
        Ok(Response::Identity(identity)) => Ok(identity),
        Ok(other) => panic!("réponse inattendue : {other:?}"),
        Err(error) => Err(error.code),
    }
}

/// Requête d'ajout d'une identité d'alias
fn alias(account: &AccountId, address: &str, is_default: bool) -> IdentityRequest {
    IdentityRequest::AddIdentity(NewIdentity {
        account: account.clone(),
        address: address.to_string(),
        display_name: "Alias".to_string(),
        signature: Some("-- alias".to_string()),
        is_default,
    })
}

/// Une nouvelle identité par défaut remplace l'ancienne : il n'y en a jamais qu'une
#[tokio::test]
async fn default_identity_is_always_unique() {
    let f = fixture::start().await;
    let id = f.add_folders_account("k").await;
    let plain = identity_response(&f, alias(&id, "plain@folders.test", false))
        .await
        .unwrap();
    assert!(!plain.is_default);
    let chosen = identity_response(&f, alias(&id, "main@folders.test", true))
        .await
        .unwrap();
    assert!(chosen.is_default);
    let list = identities(&f.engine, &id).await;
    assert_eq!(list.len(), 3);
    assert_eq!(
        list.iter().filter(|identity| identity.is_default).count(),
        1
    );
    assert_eq!(list[0].id, chosen.id);
    let back = IdentityRequest::SetDefaultIdentity {
        identity: plain.id.clone(),
    };
    assert!(identity_response(&f, back).await.unwrap().is_default);
    let list = identities(&f.engine, &id).await;
    assert_eq!(
        list.iter().filter(|identity| identity.is_default).count(),
        1
    );
    assert_eq!(list[0].id, plain.id);
}

/// Le nom et la signature se modifient ; une signature vide la supprime ; un champ absent ne change rien
#[tokio::test]
async fn update_identity_changes_name_and_signature() {
    let f = fixture::start().await;
    let id = f.add_folders_account("k").await;
    let created = identity_response(&f, alias(&id, "a@folders.test", false))
        .await
        .unwrap();
    let rename = IdentityRequest::UpdateIdentity {
        identity: created.id.clone(),
        display_name: Some("Renommée".to_string()),
        signature: None,
    };
    let renamed = identity_response(&f, rename).await.unwrap();
    assert_eq!(renamed.display_name, "Renommée");
    assert_eq!(renamed.signature.as_deref(), Some("-- alias"));
    let clear = IdentityRequest::UpdateIdentity {
        identity: created.id,
        display_name: None,
        signature: Some(String::new()),
    };
    assert_eq!(identity_response(&f, clear).await.unwrap().signature, None);
}

/// Adresse invalide, doublon d'adresse et identité inconnue donnent des codes stables
#[tokio::test]
async fn identity_errors_have_stable_codes() {
    let f = fixture::start().await;
    let id = f.add_folders_account("k").await;
    let invalid = identity_response(&f, alias(&id, "pas une adresse", false)).await;
    assert_eq!(invalid.unwrap_err(), "invalid_input.address");
    let duplicate = identity_response(&f, alias(&id, fixture::FOLDERS_USER, false)).await;
    assert_eq!(duplicate.unwrap_err(), "invalid_input.parameter");
    let ghost = IdentityRequest::SetDefaultIdentity {
        identity: IdentityId::from_raw("ghost"),
    };
    assert_eq!(
        identity_response(&f, ghost).await.unwrap_err(),
        "not_found.identity"
    );
    let unknown = alias(&AccountId::from_raw("ghost"), "x@folders.test", false);
    assert_eq!(
        identity_response(&f, unknown).await.unwrap_err(),
        "not_found.account"
    );
}
