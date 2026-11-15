#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Dépôt des souvenirs : mise à jour du seul vecteur, sous condition de texte, et péremption par dimension.

use mira_core::storage::Database;
use mira_vocabulary::{MemoryKind, MemoryRecord, MemorySource, Timestamp};

/// Un souvenir d'exemple du modèle `m1`
fn record(id: &str) -> MemoryRecord {
    MemoryRecord {
        id: id.into(),
        text: format!("texte {id}"),
        kind: MemoryKind::Fact,
        source: MemorySource::User,
        created_at: Timestamp::from_millis(1),
        model_id: Some("m1".into()),
    }
}

/// Le vecteur n'est remplacé que si le texte est toujours celui qui a été calculé
#[test]
fn update_vector_only_when_text_is_unchanged() {
    let db = Database::open_in_memory().unwrap();
    db.write(|tx| tx.insert_ai_memory(&record("a"), &[1; 8], 2))
        .unwrap();
    db.write(|tx| tx.mark_ai_memories_stale_except("m2"))
        .unwrap();
    let edited = db
        .write(|tx| tx.update_ai_memory_vector("a", "autre texte", &[9; 12], 3, "m2"))
        .unwrap();
    assert!(!edited);
    let fresh = db
        .write(|tx| tx.update_ai_memory_vector("a", "texte a", &[9; 12], 3, "m2"))
        .unwrap();
    assert!(fresh);
    assert_eq!(db.read(|tx| tx.ai_memory_vectors("m2")).unwrap().len(), 1);
}

/// Un souvenir disparu pendant le calcul n'est pas une erreur
#[test]
fn update_vector_of_a_deleted_memory_is_not_an_error() {
    let db = Database::open_in_memory().unwrap();
    let changed = db
        .write(|tx| tx.update_ai_memory_vector("gone", "x", &[0; 4], 1, "m2"))
        .unwrap();
    assert!(!changed);
}

/// Les souvenirs d'un autre nombre de dimensions sont marqués périmés, les autres restent
#[test]
fn stale_other_dimension_marks_only_mismatches() {
    let db = Database::open_in_memory().unwrap();
    db.write(|tx| tx.insert_ai_memory(&record("a"), &[0; 8], 2))
        .unwrap();
    db.write(|tx| tx.insert_ai_memory(&record("b"), &[0; 12], 3))
        .unwrap();
    let marked = db
        .write(|tx| tx.mark_ai_memories_stale_other_dim("m1", 3))
        .unwrap();
    assert_eq!(marked, 1);
    let ids: Vec<String> = db
        .read(|tx| tx.ai_memory_vectors("m1"))
        .unwrap()
        .into_iter()
        .map(|(m, _, _)| m.id)
        .collect();
    assert_eq!(ids, ["b"]);
}
