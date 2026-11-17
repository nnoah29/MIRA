#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Mémoire désactivée ou changée de modèle : refus sans effacer, souvenirs périmés comptés puis réindexés.

mod local_chat_support;
mod local_support;
mod support_ai_memory_api;
mod support_api;

use local_chat_support::fixture::LocalFixture;
use mira_core::api::{AiRequest, AiResponse};
use mira_vocabulary::{
    MemoryEmbedder, MemoryKind, MemoryRecord, MemorySettings, MemorySource, Timestamp,
};
use support_ai_memory_api::{ask, enable, listed, remember};

/// Désactive la mémoire par la façade
async fn disable(f: &LocalFixture) {
    let off = MemorySettings {
        enabled: false,
        embedder: MemoryEmbedder::Local,
    };
    ask(f, AiRequest::SetMemorySettings { settings: off })
        .await
        .unwrap();
}

/// Désactivée, la mémoire refuse de retenir et de chercher, mais laisse lister et oublier
#[tokio::test]
async fn disabled_memory_refuses_remember_and_search_but_lists_and_forgets() {
    let f = LocalFixture::start_with(300, true);
    enable(&f).await;
    let record = remember(&f, "chat").await;
    disable(&f).await;
    let again = AiRequest::RememberMemory {
        text: "x".into(),
        kind: MemoryKind::Fact,
    };
    let search = AiRequest::SearchMemories {
        query: "chat".into(),
        limit: 3,
    };
    for refused in [again, search] {
        let error = ask(&f, refused).await.unwrap_err();
        assert_eq!(error.code, "incompatible_state.ai_memory_disabled");
    }
    assert_eq!(listed(&f).await, 1);
    ask(&f, AiRequest::ForgetMemory { id: record.id })
        .await
        .unwrap();
}

/// Les souvenirs d'un autre modèle sont comptés périmés puis recalculés par la réindexation
#[tokio::test]
async fn foreign_model_memories_are_stale_until_reindexed() {
    let f = LocalFixture::start_with(300, true);
    enable(&f).await;
    remember(&f, "chat").await;
    let foreign = MemoryRecord {
        id: "old".into(),
        text: "chien".into(),
        kind: MemoryKind::Fact,
        source: MemorySource::User,
        created_at: Timestamp::from_millis(1),
        model_id: Some("ancien-modele".into()),
    };
    f.db.write(|tx| tx.insert_ai_memory(&foreign, &[0; 8], 2))
        .unwrap();
    let AiResponse::MemoryStatus(before) = ask(&f, AiRequest::GetMemoryStatus).await.unwrap()
    else {
        panic!()
    };
    assert_eq!((before.count, before.stale), (2, 1));
    let done = ask(&f, AiRequest::ReindexMemories).await.unwrap();
    assert_eq!(done, AiResponse::MemoryReindexed { done: 1, failed: 0 });
    let AiResponse::MemoryStatus(after) = ask(&f, AiRequest::GetMemoryStatus).await.unwrap() else {
        panic!()
    };
    assert_eq!(after.stale, 0);
}
