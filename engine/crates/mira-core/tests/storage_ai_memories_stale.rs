#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Dépôt des souvenirs : liste des souvenirs périmés à réindexer, par lots.

use mira_core::storage::Database;
use mira_vocabulary::{MemoryKind, MemoryRecord, MemorySource, Timestamp};

/// Un souvenir d'exemple du modèle `m1`
fn record(id: &str, at: i64) -> MemoryRecord {
    MemoryRecord {
        id: id.into(),
        text: format!("texte {id}"),
        kind: MemoryKind::Fact,
        source: MemorySource::User,
        created_at: Timestamp::from_millis(at),
        model_id: Some("m1".into()),
    }
}

/// Seuls les souvenirs périmés sont listés, au plus `limit`, les plus anciens d'abord
#[test]
fn stale_lists_only_stale_oldest_first_and_bounded() {
    let db = Database::open_in_memory().unwrap();
    for (id, at) in [("a", 1), ("b", 2), ("c", 3)] {
        db.write(|tx| tx.insert_ai_memory(&record(id, at), &[0; 8], 2))
            .unwrap();
    }
    assert!(db.read(|tx| tx.stale_ai_memories(10)).unwrap().is_empty());
    db.write(|tx| tx.mark_ai_memories_stale_except("m2"))
        .unwrap();
    let ids: Vec<String> = db
        .read(|tx| tx.stale_ai_memories(2))
        .unwrap()
        .into_iter()
        .map(|m| m.id)
        .collect();
    assert_eq!(ids, ["a", "b"]);
}
