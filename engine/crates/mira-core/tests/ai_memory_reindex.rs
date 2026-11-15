#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Changement de modèle d'embeddings : les vecteurs d'un autre modèle sont ignorés puis recalculés.

mod support_ai_memory;

use mira_vocabulary::{MemoryKind, MemorySource};
use support_ai_memory::{embedder, setup, store_over};

/// Après un changement de modèle, les anciens souvenirs ne sont plus rendus tant qu'ils ne sont pas réindexés
#[tokio::test]
async fn recall_ignores_stale_vectors_of_other_model() {
    let s = setup("m1");
    s.store
        .remember("chat noir", MemoryKind::Fact, MemorySource::User)
        .await
        .unwrap();
    let other = store_over(&s.db, &embedder("m2"));
    assert!(other.recall("chat", 5).await.unwrap().is_empty());
}

/// La réindexation recalcule tout avec le nouveau modèle et les souvenirs redeviennent trouvables
#[tokio::test]
async fn reindex_recomputes_stale_with_new_model() {
    let s = setup("m1");
    for text in ["chat noir", "chien gris", "mail du lundi"] {
        s.store
            .remember(text, MemoryKind::Fact, MemorySource::User)
            .await
            .unwrap();
    }
    let other = store_over(&s.db, &embedder("m2"));
    assert_eq!(other.reindex().await.unwrap(), 3);
    let hits = other.recall("chat", 5).await.unwrap();
    assert_eq!(hits[0].record.text, "chat noir");
    assert_eq!(hits[0].record.model_id.as_deref(), Some("m2"));
    assert_eq!(other.reindex().await.unwrap(), 0);
}
