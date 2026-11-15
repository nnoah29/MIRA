#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Service de mémoire : se souvenir, retrouver par le sens, oublier ; rien n'est stocké si l'embedder échoue.

mod support_ai_memory;

use std::sync::atomic::Ordering;

use mira_vocabulary::{MemoryKind, MemorySource};
use support_ai_memory::setup;

/// Un souvenir retrouvé par une question de même sens, avec son genre et son origine
#[tokio::test]
async fn remember_then_recall_finds_it() {
    let s = setup("m1");
    s.store
        .remember(
            "Mon chat s'appelle Mochi",
            MemoryKind::Fact,
            MemorySource::User,
        )
        .await
        .unwrap();
    s.store
        .remember("Mon chien aime le parc", MemoryKind::Fact, MemorySource::Ai)
        .await
        .unwrap();
    let hits = s.store.recall("où est mon chat ?", 5).await.unwrap();
    assert_eq!(hits[0].record.text, "Mon chat s'appelle Mochi");
    assert_eq!(hits[0].record.source, MemorySource::User);
    assert_eq!(hits[0].record.model_id.as_deref(), Some("m1"));
}

/// Un souvenir oublié ne sort plus de la recherche
#[tokio::test]
async fn forget_removes_from_recall() {
    let s = setup("m1");
    let kept = s
        .store
        .remember("chat noir", MemoryKind::Fact, MemorySource::User)
        .await
        .unwrap();
    s.store.forget(&kept.id).await.unwrap();
    assert!(s.store.recall("chat", 5).await.unwrap().is_empty());
}

/// Sans souvenir, la recherche ne sollicite pas l'embedder
#[tokio::test]
async fn recall_with_no_memories_is_empty_without_calling_embedder() {
    let s = setup("m1");
    assert!(s.store.recall("chat", 5).await.unwrap().is_empty());
    assert_eq!(s.embedder.calls.load(Ordering::SeqCst), 0);
}

/// Si l'embedder échoue, rien n'est stocké
#[tokio::test]
async fn embedder_failure_stores_nothing() {
    let s = setup("m1");
    s.embedder.fail.store(true, Ordering::SeqCst);
    let result = s
        .store
        .remember("chat", MemoryKind::Fact, MemorySource::User)
        .await;
    assert!(result.is_err());
    assert!(
        s.db.read(|tx| tx.list_ai_memories(10, None))
            .unwrap()
            .is_empty()
    );
}

/// Tout oublier rend le nombre de souvenirs effacés
#[tokio::test]
async fn forget_all_returns_count() {
    let s = setup("m1");
    for text in ["chat", "chien"] {
        s.store
            .remember(text, MemoryKind::Action, MemorySource::User)
            .await
            .unwrap();
    }
    assert_eq!(s.store.forget_all().await.unwrap(), 2);
}
