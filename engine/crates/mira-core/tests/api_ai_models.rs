#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Domaine « Ai » par la façade : consentement, test de connexion, liste des modèles et son cache.

mod ai_models_support;
mod support_ai_api;
mod support_api;

use mira_core::api::{AiRequest, AiResponse};
use mira_testkit::ai::Behavior;
use mira_vocabulary::Timestamp;
use support_ai_api::calls::{add, call, list};
use support_ai_api::operations::{consent, models, set_key, update};
use support_ai_api::providers::{EDITABLE, started};

const ONE_MODEL: &str = r#"{"data":[{"id":"model-a","created":100}]}"#;
const OTHER_MODEL: &str = r#"{"data":[{"id":"model-c","created":300}]}"#;

/// Met cette réponse de liste de modèles en file sur le faux serveur
fn queue_models(f: &ai_models_support::ModelsFixture, json: &str) {
    f.server.queue(Behavior::Models {
        json: json.to_string(),
    });
}

/// Le consentement note la date du moteur, une seule fois
#[tokio::test]
async fn acknowledge_consent_sets_consent_at() {
    let f = started().await;
    let created = add(&f.engine, EDITABLE, "A", Some(&f.server.base_url()))
        .await
        .unwrap();
    let given = consent(&f.engine, &created.id).await;
    assert_eq!(
        given.consent_at,
        Some(Timestamp::from_millis(1_700_000_000_000))
    );
    assert_eq!(consent(&f.engine, &created.id).await, given);
}

/// Tester une connexion interroge toujours le service, garde la liste et rend `Done`
#[tokio::test]
async fn test_connection_refreshes_models_and_returns_done() {
    let f = started().await;
    let created = add(&f.engine, EDITABLE, "A", Some(&f.server.base_url()))
        .await
        .unwrap();
    set_key(&f.engine, &created.id, "sk-test").await.unwrap();
    queue_models(&f, ONE_MODEL);
    queue_models(&f, OTHER_MODEL);
    let request = AiRequest::TestAiConnection {
        connection: created.id.clone(),
    };
    assert_eq!(
        call(&f.engine, request.clone()).await.unwrap(),
        AiResponse::Done
    );
    assert_eq!(call(&f.engine, request).await.unwrap(), AiResponse::Done);
    assert_eq!(f.server.requests().len(), 2);
    let kept = list(&f.engine).await.remove(0).models.unwrap();
    assert_eq!(kept.models[0].id, "model-c");
}

/// La liste gardée sert tant qu'on ne rafraîchit pas, même relue fraîchement en base
#[tokio::test]
async fn list_models_uses_cache_unless_refresh() {
    let f = started().await;
    let created = add(&f.engine, EDITABLE, "A", Some(&f.server.base_url()))
        .await
        .unwrap();
    set_key(&f.engine, &created.id, "sk-test").await.unwrap();
    queue_models(&f, ONE_MODEL);
    queue_models(&f, OTHER_MODEL);
    let first = models(&f.engine, &created.id, false).await.unwrap();
    let cached = models(&f.engine, &created.id, false).await.unwrap();
    assert_eq!(cached, first);
    assert_eq!(f.server.requests().len(), 1);
    let fresh = models(&f.engine, &created.id, true).await.unwrap();
    assert_eq!(fresh.models[0].id, "model-c");
    assert_eq!(f.server.requests().len(), 2);
}

/// Le consentement vaut pour une destination : changer l'adresse le remet à zéro, la garder non
#[tokio::test]
async fn changing_the_url_resets_consent() {
    let f = started().await;
    let url = f.server.base_url();
    let created = add(&f.engine, EDITABLE, "A", Some(&url)).await.unwrap();
    consent(&f.engine, &created.id).await;
    let same = update(&f.engine, &created.id, "A", Some(&url), None)
        .await
        .unwrap();
    assert!(same.consent_at.is_some());
    let moved = update(
        &f.engine,
        &created.id,
        "A",
        Some("http://127.0.0.1:9/v1"),
        None,
    )
    .await
    .unwrap();
    assert_eq!(moved.consent_at, None);
    assert_eq!(list(&f.engine).await[0].consent_at, None);
}

/// Une liste de modèles gardée illisible ne fait pas échouer la liste des connexions
#[tokio::test]
async fn list_degrades_when_cached_models_are_corrupted() {
    let f = started().await;
    let healthy = add(&f.engine, EDITABLE, "A", Some(&f.server.base_url()))
        .await
        .unwrap();
    let broken = add(&f.engine, EDITABLE, "B", Some(&f.server.base_url()))
        .await
        .unwrap();
    set_key(&f.engine, &healthy.id, "sk-test").await.unwrap();
    queue_models(&f, ONE_MODEL);
    models(&f.engine, &healthy.id, true).await.unwrap();
    f.corrupt_models(broken.id.as_str());
    let listed = list(&f.engine).await;
    assert_eq!(listed.len(), 2);
    assert!(listed[0].models.is_some());
    assert!(listed[1].models.is_none());
}
