#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Domaine « Ai » par la façade : fournisseurs, création, défaut, suppression, mise à jour.

mod ai_models_support;
mod support_ai_api;
mod support_api;

use mira_core::api::{AiRequest, AiResponse};
use mira_core::ports::SecretStore;
use mira_vocabulary::{AiConnectionId, AiSecretRef, ProfileId};
use support_ai_api::calls::{add, call, list};
use support_ai_api::operations::{consent, requests_on, set_default, set_key, update};
use support_ai_api::providers::{EDITABLE, FIXED, started};

/// Les fournisseurs rendus sont ceux du registre, dans l'ordre d'enregistrement
#[tokio::test]
async fn list_providers_returns_registered_descriptors_in_order() {
    let f = started().await;
    let answer = call(&f.engine, AiRequest::ListAiProviders).await.unwrap();
    let AiResponse::Providers(found) = answer else {
        panic!("réponse inattendue");
    };
    let ids: Vec<&str> = found.iter().map(|item| item.id.0.as_str()).collect();
    assert_eq!(ids, [EDITABLE, FIXED]);
    assert!(found[0].base_url_editable && !found[1].base_url_editable);
}

/// La connexion créée apparaît dans la liste, sans clé, ni champ qui en porte une
#[tokio::test]
async fn add_then_list_shows_connection_without_key_field() {
    let f = started().await;
    let created = add(
        &f.engine,
        EDITABLE,
        "  Mon IA ",
        Some("http://127.0.0.1:9/v1"),
    )
    .await
    .unwrap();
    assert_eq!(created.label, "Mon IA");
    assert!(!created.has_key && created.models.is_none() && created.consent_at.is_none());
    let listed = list(&f.engine).await;
    assert_eq!(listed, vec![created]);
    let json = serde_json::to_value(&listed[0]).unwrap();
    let names: Vec<&String> = json.as_object().unwrap().keys().collect();
    assert!(
        names
            .iter()
            .all(|name| !name.contains("api") && *name != "key")
    );
}

/// Aucune connexion ne devient défaut toute seule ; la désigner retire l'ancien défaut
#[tokio::test]
async fn first_connection_is_not_default_until_set() {
    let f = started().await;
    let first = add(&f.engine, FIXED, "A", None).await.unwrap();
    let second = add(&f.engine, FIXED, "B", None).await.unwrap();
    assert!(!first.is_default && !second.is_default);
    assert!(set_default(&f.engine, &first.id).await.is_default);
    set_default(&f.engine, &second.id).await;
    let flags: Vec<bool> = list(&f.engine).await.iter().map(|c| c.is_default).collect();
    assert_eq!(flags, [false, true]);
}

/// Supprimer une connexion supprime sa clé ; si elle était le défaut, aucune autre ne le devient
#[tokio::test]
async fn remove_deletes_the_stored_key() {
    let f = started().await;
    let first = add(&f.engine, FIXED, "A", None).await.unwrap();
    let second = add(&f.engine, FIXED, "B", None).await.unwrap();
    set_default(&f.engine, &first.id).await;
    set_key(&f.engine, &first.id, "sk-secret-1").await.unwrap();
    let location = AiSecretRef {
        profile: ProfileId::from_raw("default"),
        connection: first.id.clone(),
    };
    assert!(f.secrets.get(&location).unwrap().is_some());
    let request = AiRequest::RemoveAiConnection {
        connection: first.id.clone(),
    };
    assert_eq!(call(&f.engine, request).await.unwrap(), AiResponse::Done);
    assert!(f.secrets.get(&location).unwrap().is_none());
    let rest = list(&f.engine).await;
    assert_eq!(rest.len(), 1);
    assert_eq!(rest[0].id, second.id);
    assert!(!rest[0].is_default);
}

/// La mise à jour change libellé, modèle par défaut et adresse, mais pas défaut, consentement ni modèles
#[tokio::test]
async fn update_changes_label_url_and_model_only() {
    let f = started().await;
    let url = "http://127.0.0.1:9/v1";
    let created = add(&f.engine, EDITABLE, "A", Some(url)).await.unwrap();
    set_default(&f.engine, &created.id).await;
    consent(&f.engine, &created.id).await;
    let renamed = update(
        &f.engine,
        &created.id,
        " Renommée ",
        Some(url),
        Some("model-b"),
    )
    .await
    .unwrap();
    assert_eq!(renamed.label, "Renommée");
    assert_eq!(renamed.default_model.as_deref(), Some("model-b"));
    assert!(renamed.is_default && renamed.consent_at.is_some());
    let moved = update(
        &f.engine,
        &created.id,
        "Renommée",
        Some("http://localhost:9/v2/"),
        None,
    )
    .await
    .unwrap();
    assert_eq!(moved.base_url.as_deref(), Some("http://localhost:9/v2"));
    assert_eq!(moved.default_model, None);
}

/// Toute opération sur une connexion inconnue rend `not_found.ai_connection`
#[tokio::test]
async fn unknown_connection_is_not_found() {
    let f = started().await;
    let connection = AiConnectionId::from_raw("absente");
    for request in requests_on(&connection) {
        let error = call(&f.engine, request).await.unwrap_err();
        assert_eq!(error, "not_found.ai_connection");
    }
    let keyed = set_key(&f.engine, &connection, "sk-x").await;
    assert_eq!(keyed.unwrap_err(), "not_found.ai_connection");
    let changed = update(&f.engine, &connection, "A", None, None).await;
    assert_eq!(changed.unwrap_err(), "not_found.ai_connection");
}
