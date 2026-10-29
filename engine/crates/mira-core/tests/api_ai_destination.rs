#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Domaine « Ai » par la façade : changer l'adresse oublie clé, consentement et modèles ; le reste les garde.

mod ai_models_support;
mod support_ai_api;
mod support_api;

use std::sync::atomic::Ordering;

use mira_core::api::AiRequest;
use mira_core::ports::SecretStore;
use mira_vocabulary::{AiConnection, AiSecretRef, ProfileId};
use support_ai_api::calls::{call, list};
use support_ai_api::destination::prepared;
use support_ai_api::operations::update;
use support_ai_api::providers::{started, started_with_flaky_delete};

const OTHER_URL: &str = "http://127.0.0.1:9/v1";

/// Emplacement de la clé de cette connexion dans le trousseau
fn location(connection: &AiConnection) -> AiSecretRef {
    AiSecretRef {
        profile: ProfileId::from_raw("default"),
        connection: connection.id.clone(),
    }
}

/// Une autre adresse efface la clé du trousseau : elle ne sert jamais auprès d'un autre serveur
#[tokio::test]
async fn changing_the_url_clears_the_key() {
    let f = started().await;
    let ready = prepared(&f).await;
    assert!(f.secrets.get(&location(&ready)).unwrap().is_some());
    let moved = update(&f.engine, &ready.id, "A", Some(OTHER_URL), None).await;
    assert!(!moved.unwrap().has_key);
    assert!(f.secrets.get(&location(&ready)).unwrap().is_none());
    assert!(!list(&f.engine).await[0].has_key);
}

/// Une autre adresse oublie la liste de modèles gardée, en réponse comme en base
#[tokio::test]
async fn changing_the_url_clears_cached_models() {
    let f = started().await;
    let ready = prepared(&f).await;
    let moved = update(&f.engine, &ready.id, "A", Some(OTHER_URL), None).await;
    assert_eq!(moved.unwrap().models, None);
    assert_eq!(f.stored(&ready).models, None);
    assert_eq!(f.stored(&ready).consent_at, None);
}

/// La même adresse, même écrite avec un `/` final, garde clé, consentement et modèles ; libellé et modèle changent
#[tokio::test]
async fn unchanged_url_keeps_key_consent_and_models() {
    let f = started().await;
    let ready = prepared(&f).await;
    let same = format!("{}/", ready.base_url.clone().unwrap());
    let kept = update(&f.engine, &ready.id, "B", Some(&same), Some("model-a"))
        .await
        .unwrap();
    assert_eq!(kept.label, "B");
    assert!(kept.has_key);
    assert_eq!(kept.consent_at, ready.consent_at);
    assert_eq!(kept.models, ready.models);
    assert!(f.secrets.get(&location(&ready)).unwrap().is_some());
}

/// Si la clé ne peut pas être supprimée, changer l'adresse n'écrit rien : adresse, consentement, modèles et clé restent
#[tokio::test]
async fn url_change_with_failing_key_delete_changes_nothing() {
    let (f, fail) = started_with_flaky_delete().await;
    let ready = prepared(&f).await;
    fail.store(true, Ordering::SeqCst);
    let refused = update(&f.engine, &ready.id, "B", Some(OTHER_URL), None).await;
    assert_eq!(refused.unwrap_err(), "secrets.keyring_unavailable");
    let stored = f.stored(&ready);
    assert_eq!(stored.base_url, ready.base_url);
    assert_eq!(stored.label, "A");
    assert_eq!(stored.consent_at, ready.consent_at);
    assert_eq!(stored.models, ready.models);
    assert!(f.secrets.get(&location(&ready)).unwrap().is_some());
    fail.store(false, Ordering::SeqCst);
    let retried = update(&f.engine, &ready.id, "B", Some(OTHER_URL), None).await;
    assert!(!retried.unwrap().has_key);
}

/// Si la clé ne peut pas être supprimée, la connexion reste, avec sa clé : rien n'est orphelin
#[tokio::test]
async fn remove_with_failing_key_delete_keeps_the_connection() {
    let (f, fail) = started_with_flaky_delete().await;
    let ready = prepared(&f).await;
    fail.store(true, Ordering::SeqCst);
    let request = AiRequest::RemoveAiConnection {
        connection: ready.id.clone(),
    };
    let refused = call(&f.engine, request).await;
    assert_eq!(refused.unwrap_err(), "secrets.keyring_unavailable");
    assert_eq!(list(&f.engine).await.len(), 1);
    assert!(f.secrets.get(&location(&ready)).unwrap().is_some());
}
