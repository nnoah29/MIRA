#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Domaine « Ai » par la façade : changer l'adresse oublie clé, consentement et modèles ; le reste les garde.

mod ai_models_support;
mod support_ai_api;
mod support_api;

use mira_core::ports::SecretStore;
use mira_vocabulary::{AiConnection, AiSecretRef, ProfileId};
use support_ai_api::calls::list;
use support_ai_api::destination::prepared;
use support_ai_api::operations::update;
use support_ai_api::providers::started;

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
