#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Embedder local : serveur dédié lancé à la demande en mode embeddings, réutilisé, sans réglage de discussion.

mod local_support;

use std::sync::Arc;

use local_support::runtime::RuntimeFixture;
use local_support::test_model_named;
use mira_core::ai::local::LocalRuntimeManager;
use mira_core::ai::memory::{Embedder, LocalEmbedder, LocalEmbedderSetup, embedding_runtime};

/// Un embedder dont le serveur est celui du faux dossier, avec les arguments d'embeddings donnés
fn embedder_for(fixture: &RuntimeFixture) -> LocalEmbedder {
    let extra: Vec<String> = ["--embedding", "--pooling", "cls"]
        .iter()
        .map(|a| (*a).to_string())
        .collect();
    LocalEmbedder::new(LocalEmbedderSetup {
        manager: LocalRuntimeManager::new(),
        runtime: embedding_runtime(&fixture.runtime, &extra),
        executable: fixture.dir.path().join("fake-server"),
        run_dir: fixture.run_dir(),
        models_dir: fixture.dir.path().join("models"),
        resources: Arc::new(fixture.resources),
        model: test_model_named(fixture, "mini"),
    })
}

/// Le premier calcul lance le serveur avec les arguments d'embeddings et sans ceux de discussion
#[tokio::test]
async fn first_embed_launches_server_in_embedding_mode() {
    let mut fixture = RuntimeFixture::new(&[]);
    fixture.runtime.reasoning_off_args = vec!["--reasoning-budget".into(), "0".into()];
    let embedder = embedder_for(&fixture);
    let texts = vec!["a".to_string(), "b".to_string()];
    let vectors = embedder.embed(&texts).await.unwrap();
    assert_eq!(vectors, vec![vec![1.0, 0.0], vec![1.0, 1.0]]);
    let argv = fixture.starts()[0]["argv"].to_string();
    assert!(argv.contains("--embedding") && argv.contains("cls"));
    assert!(!argv.contains("--reasoning-budget"));
    embedder.stop().await;
}

/// Le second calcul réutilise le serveur déjà lancé
#[tokio::test]
async fn second_embed_reuses_the_running_server() {
    let fixture = RuntimeFixture::new(&[]);
    let embedder = embedder_for(&fixture);
    embedder.embed(&["a".to_string()]).await.unwrap();
    embedder.embed(&["b".to_string()]).await.unwrap();
    assert_eq!(fixture.starts().len(), 1);
    embedder.stop().await;
}

/// L'identifiant du modèle est celui de la fiche
#[tokio::test]
async fn model_id_is_the_pinned_model_id() {
    let fixture = RuntimeFixture::new(&[]);
    assert_eq!(embedder_for(&fixture).model_id(), "mini");
}
