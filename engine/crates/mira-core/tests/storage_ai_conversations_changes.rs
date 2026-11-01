#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Dépôt des conversations d'IA : renommage, suppression et effacement total.

mod support_ai_conversation_rows;

use mira_core::storage::Database;
use mira_vocabulary::Timestamp;
use support_ai_conversation_rows::{listed_ids, save};

/// Renommer change le titre sans toucher la date de mise à jour
#[test]
fn rename_keeps_updated_at() {
    let db = Database::open_in_memory().unwrap();
    save(&db, "c-1", "Avant", 10);
    let meta = db
        .write(|tx| tx.rename_ai_conversation("c-1", "Après"))
        .unwrap();
    assert_eq!(meta.title, "Après");
    assert_eq!(meta.updated_at, Timestamp::from_millis(10));
    let refused = db.write(|tx| tx.rename_ai_conversation("absent", "X"));
    assert_eq!(refused.unwrap_err().code, "not_found.ai_conversation");
}

/// Supprimer une conversation inconnue échoue, une connue ne retire que celle-là
#[test]
fn delete_removes_only_that_one_and_refuses_unknown() {
    let db = Database::open_in_memory().unwrap();
    save(&db, "a", "A", 1);
    save(&db, "b", "B", 2);
    db.write(|tx| tx.delete_ai_conversation("a")).unwrap();
    assert_eq!(listed_ids(&db), ["b"]);
    let refused = db.write(|tx| tx.delete_ai_conversation("a"));
    assert_eq!(refused.unwrap_err().code, "not_found.ai_conversation");
}

/// Tout effacer vide la table, même vide
#[test]
fn clear_removes_all() {
    let db = Database::open_in_memory().unwrap();
    save(&db, "a", "A", 1);
    save(&db, "b", "B", 2);
    db.write(|tx| tx.clear_ai_conversations()).unwrap();
    assert!(listed_ids(&db).is_empty());
    db.write(|tx| tx.clear_ai_conversations()).unwrap();
}
