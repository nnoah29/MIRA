#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Changement de modèle d'embeddings : vecteurs d'un autre modèle ignorés, recalculés par lots, sans jamais boucler.

mod support_ai_memory;

use std::sync::atomic::Ordering;
use std::time::Duration;

use mira_vocabulary::{MemoryKind, MemoryRecord, MemorySource, Timestamp};
use support_ai_memory::{Setup, embedder, setup, store_over};

/// Retient ces textes avec l'embedder du service
async fn remember_all(s: &Setup, texts: &[&str]) {
    for text in texts {
        s.store
            .remember(text, MemoryKind::Fact, MemorySource::User)
            .await
            .unwrap();
    }
}

/// Après un changement de modèle, les anciens souvenirs ne sont plus rendus tant qu'ils ne sont pas réindexés
#[tokio::test]
async fn recall_ignores_stale_vectors_of_other_model() {
    let s = setup("m1");
    remember_all(&s, &["chat noir"]).await;
    let other = store_over(&s.db, &embedder("m2"));
    assert!(other.recall("chat", 5).await.unwrap().is_empty());
}

/// La réindexation recalcule tout avec le nouveau modèle et les souvenirs redeviennent trouvables
#[tokio::test]
async fn reindex_recomputes_stale_with_new_model() {
    let s = setup("m1");
    remember_all(&s, &["chat noir", "chien gris", "mail du lundi"]).await;
    let other = store_over(&s.db, &embedder("m2"));
    let report = other.reindex().await.unwrap();
    assert_eq!((report.done, report.failed), (3, 0));
    let hits = other.recall("chat", 5).await.unwrap();
    assert_eq!(hits[0].record.model_id.as_deref(), Some("m2"));
    assert_eq!(other.reindex().await.unwrap().done, 0);
}

/// Un embedder qui ne rend aucun vecteur ne fait pas tourner la réindexation sans fin
#[tokio::test]
async fn reindex_terminates_when_embedder_returns_no_vector() {
    let s = setup("m1");
    remember_all(&s, &["chat noir", "chien gris"]).await;
    let broken = embedder("m2");
    broken.short.store(true, Ordering::SeqCst);
    let other = store_over(&s.db, &broken);
    let report = tokio::time::timeout(Duration::from_secs(5), other.reindex())
        .await
        .unwrap()
        .unwrap();
    assert_eq!((report.done, report.failed), (0, 2));
}

/// Un texte que le modèle refuse ne bloque pas les autres : ils sont réindexés, lui est compté en échec
#[tokio::test]
async fn reindex_skips_the_text_the_embedder_refuses() {
    let s = setup("m1");
    remember_all(&s, &["chat noir", "chien gris"]).await;
    let refused = MemoryRecord {
        id: "refused".into(),
        text: "texte poison".into(),
        kind: MemoryKind::Fact,
        source: MemorySource::User,
        created_at: Timestamp::from_millis(9_000),
        model_id: Some("m1".into()),
    };
    s.db.write(|tx| tx.insert_ai_memory(&refused, &[0; 16], 4))
        .unwrap();
    let other = store_over(&s.db, &embedder("m2"));
    let report = other.reindex().await.unwrap();
    assert_eq!((report.done, report.failed), (2, 1));
    assert_eq!(
        other.recall("chat", 5).await.unwrap()[0].record.text,
        "chat noir"
    );
}

/// Un même modèle qui change de dimension périme ses vecteurs, qui reviennent après réindexation
#[tokio::test]
async fn recall_marks_vectors_of_another_dimension_stale() {
    let s = setup("m1");
    remember_all(&s, &["chat noir"]).await;
    let wide = embedder("m1");
    wide.wide.store(true, Ordering::SeqCst);
    let other = store_over(&s.db, &wide);
    assert!(other.recall("chat", 5).await.unwrap().is_empty());
    assert_eq!(other.reindex().await.unwrap().done, 1);
    assert_eq!(other.recall("chat", 5).await.unwrap().len(), 1);
}
