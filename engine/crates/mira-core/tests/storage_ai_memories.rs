#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Dépôt des souvenirs de l'IA : insertion, liste, vecteurs par modèle, mise à jour et suppression.

use mira_core::storage::Database;
use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{MemoryKind, MemoryRecord, MemorySource, Timestamp};

/// Un souvenir d'exemple indexé par ce modèle à cet instant
fn record(id: &str, model: &str, at: i64) -> MemoryRecord {
    MemoryRecord {
        id: id.into(),
        text: format!("texte {id}"),
        kind: MemoryKind::Fact,
        source: MemorySource::User,
        created_at: Timestamp::from_millis(at),
        model_id: Some(model.into()),
    }
}

/// La liste va du plus récent au plus ancien
#[test]
fn insert_then_list_returns_newest_first() {
    let db = Database::open_in_memory().unwrap();
    for (id, at) in [("a", 1), ("b", 2), ("c", 3)] {
        db.write(|tx| tx.insert_ai_memory(&record(id, "m1", at), &[0; 8], 2))
            .unwrap();
    }
    let ids: Vec<String> = db
        .read(|tx| tx.list_ai_memories(10, None))
        .unwrap()
        .into_iter()
        .map(|m| m.id)
        .collect();
    assert_eq!(ids, ["c", "b", "a"]);
}

/// Les vecteurs rendus sont ceux du modèle demandé et jamais ceux marqués périmés
#[test]
fn vectors_exclude_stale_and_other_models() {
    let db = Database::open_in_memory().unwrap();
    db.write(|tx| tx.insert_ai_memory(&record("old", "m1", 1), &[1; 8], 2))
        .unwrap();
    db.write(|tx| tx.insert_ai_memory(&record("new", "m2", 2), &[2; 8], 2))
        .unwrap();
    let stale = db
        .write(|tx| tx.mark_ai_memories_stale_except("m2"))
        .unwrap();
    assert_eq!(stale, 1);
    let found = db.read(|tx| tx.ai_memory_vectors("m2")).unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].0.id, "new");
    assert!(db.read(|tx| tx.ai_memory_vectors("m1")).unwrap().is_empty());
}

/// Réindexer un texte remplace son vecteur et le rend de nouveau cherchable
#[test]
fn update_text_replaces_vector_and_clears_stale() {
    let db = Database::open_in_memory().unwrap();
    db.write(|tx| tx.insert_ai_memory(&record("a", "m1", 1), &[1; 8], 2))
        .unwrap();
    db.write(|tx| tx.mark_ai_memories_stale_except("m2"))
        .unwrap();
    db.write(|tx| tx.update_ai_memory_text("a", "nouveau", &[9; 12], 3, "m2"))
        .unwrap();
    let found = db.read(|tx| tx.ai_memory_vectors("m2")).unwrap();
    assert_eq!(found[0].0.text, "nouveau");
    assert_eq!(found[0].1, vec![9; 12]);
    assert_eq!(found[0].2, 3);
}

/// Supprimer un souvenir inconnu donne `not_found.ai_memory`
#[test]
fn delete_unknown_is_not_found() {
    let db = Database::open_in_memory().unwrap();
    let error = db.write(|tx| tx.delete_ai_memory("nope")).unwrap_err();
    assert_eq!(error.code, not_found::AI_MEMORY.0);
}

/// Tout effacer rend le nombre de souvenirs supprimés et vide la liste
#[test]
fn delete_all_returns_count() {
    let db = Database::open_in_memory().unwrap();
    for id in ["a", "b"] {
        db.write(|tx| tx.insert_ai_memory(&record(id, "m1", 1), &[0; 8], 2))
            .unwrap();
    }
    assert_eq!(db.write(|tx| tx.delete_all_ai_memories()).unwrap(), 2);
    assert!(
        db.read(|tx| tx.list_ai_memories(10, None))
            .unwrap()
            .is_empty()
    );
}
