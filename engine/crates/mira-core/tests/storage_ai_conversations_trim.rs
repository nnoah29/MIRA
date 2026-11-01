#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Dépôt des conversations d'IA : rétention et migration 20.

mod support_ai_conversation_rows;

use mira_core::storage::{Database, schema_version};
use support_ai_conversation_rows::{listed_ids, save};

/// L'éviction garde les plus récentes et ne supprime jamais la conversation protégée
#[test]
fn trim_keeps_newest_and_never_the_protected_one() {
    let db = Database::open_in_memory().unwrap();
    for (id, now) in [("old", 1), ("b", 2), ("c", 3), ("d", 4), ("e", 5)] {
        save(&db, id, id, now);
    }
    let removed = db.write(|tx| tx.trim_ai_conversations(2, "old")).unwrap();
    assert_eq!(removed, 2);
    assert_eq!(listed_ids(&db), ["e", "d", "old"]);
}

/// La migration 20 s'applique à un profil existant en version 19
#[test]
fn migration_20_applies_on_existing_profile() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mira.sqlite3");
    drop(Database::open(&path).unwrap());
    let raw = rusqlite::Connection::open(&path).unwrap();
    raw.execute_batch("DROP TABLE ai_conversations; PRAGMA user_version = 19;")
        .unwrap();
    drop(raw);
    let db = Database::open(&path).unwrap();
    assert_eq!(schema_version(), 20);
    save(&db, "c-1", "A", 1);
    assert_eq!(listed_ids(&db), ["c-1"]);
}
