#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Mémoire côté moteur, mode fournisseur : clé et adresse de la connexion, consentement et type « mémoire » exigés.

mod ai_models_support;
mod support_ai_api;
mod support_api;

use mira_testkit::ai::Behavior;
use mira_vocabulary::error::codes::incompatible_state::{
    AI_CONSENT_REQUIRED, AI_CONTENT_NOT_ALLOWED,
};
use mira_vocabulary::{
    AiConnection, ContentKind, MemoryEmbedder, MemoryKind, MemorySettings, MemorySource,
};
use support_ai_api::calls::add;
use support_ai_api::operations::{consent, set_key};
use support_ai_api::privacy::set_allowed;
use support_ai_api::providers::{EDITABLE, started};

use ai_models_support::ModelsFixture;

const REMOTE: &str = "https://embeddings.provider.test/v1";

/// Active la mémoire en mode fournisseur sur cette connexion
fn enable_provider(f: &ModelsFixture, connection: &AiConnection) {
    let settings = MemorySettings {
        enabled: true,
        embedder: MemoryEmbedder::Provider {
            connection: connection.id.clone(),
            model: "embed-1".into(),
        },
    };
    f.db.write(|tx| tx.set_ai_memory_settings(&settings))
        .unwrap();
}

/// Connexion sur cette adresse, avec clé et consentement, sans interroger la liste des modèles
async fn ready(f: &ModelsFixture, url: &str) -> AiConnection {
    let created = add(&f.engine, EDITABLE, "A", Some(url)).await.unwrap();
    set_key(&f.engine, &created.id, "sk-destination")
        .await
        .unwrap();
    consent(&f.engine, &created.id).await;
    created
}

/// Réponse d'embeddings à un seul texte
fn one_vector() -> Behavior {
    Behavior::Status {
        code: 200,
        headers: vec![],
        body: r#"{"data":[{"index":0,"embedding":[1.0,0.0]}]}"#.to_string(),
    }
}

/// Avec consentement et type « mémoire » autorisé, le texte part avec la clé de la connexion, sous un modèle qualifié
#[tokio::test]
async fn provider_mode_embeds_with_the_connection_key() {
    let f = started().await;
    let connection = ready(&f, &f.server.base_url()).await;
    set_allowed(
        &f.engine,
        &connection.id,
        vec![ContentKind::MailMessage, ContentKind::Memory],
    )
    .await
    .unwrap();
    enable_provider(&f, &connection);
    f.server.queue(one_vector());
    let store = f.engine.memory_store().await.unwrap();
    let record = store
        .remember("chat noir", MemoryKind::Fact, MemorySource::User)
        .await
        .unwrap();
    assert_eq!(record.model_id.as_deref(), Some("test-provider:embed-1"));
    let seen = f.server.requests().pop().unwrap();
    assert_eq!(seen.path, "/embeddings");
    assert!(
        seen.header("authorization")
            .unwrap()
            .ends_with("sk-destination")
    );
}

/// Vers une adresse distante, sans le type « mémoire » autorisé, le texte ne part pas
#[tokio::test]
async fn provider_mode_without_memory_kind_is_refused() {
    let f = started().await;
    let connection = ready(&f, REMOTE).await;
    enable_provider(&f, &connection);
    let error = f.engine.memory_store().await.unwrap_err();
    assert_eq!(error.code, AI_CONTENT_NOT_ALLOWED.0);
    assert!(f.server.requests().is_empty());
}

/// Vers une adresse distante, sans consentement à la connexion, le texte ne part pas
#[tokio::test]
async fn provider_mode_without_consent_is_refused() {
    let f = started().await;
    let connection = add(&f.engine, EDITABLE, "A", Some(REMOTE)).await.unwrap();
    enable_provider(&f, &connection);
    let error = f.engine.memory_store().await.unwrap_err();
    assert_eq!(error.code, AI_CONSENT_REQUIRED.0);
    assert!(f.server.requests().is_empty());
}
