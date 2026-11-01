#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Dépôt des conversations d'IA : enregistrement, liste paginée et lecture complète.

mod support_ai_conversation_rows;

use mira_core::storage::Database;
use mira_vocabulary::Timestamp;
use support_ai_conversation_rows::{listed_ids, save};

/// Enregistrer deux fois le même identifiant garde une ligne, la date de création et remplace le reste
#[test]
fn save_twice_keeps_one_row_and_created_at() {
    let db = Database::open_in_memory().unwrap();
    save(&db, "c-1", "Avant", 10);
    let second = db
        .write(|tx| tx.save_ai_conversation("c-1", "Après", 2, "{\"a\":1}", 20))
        .unwrap();
    assert_eq!(listed_ids(&db), ["c-1"]);
    assert_eq!(second.created_at, Timestamp::from_millis(10));
    assert_eq!(second.updated_at, Timestamp::from_millis(20));
    let found = db.read(|tx| tx.get_ai_conversation("c-1")).unwrap();
    assert_eq!(found.meta.title, "Après");
    assert_eq!(found.meta.payload_version, 2);
    assert_eq!(found.payload, "{\"a\":1}");
}

/// La liste va du plus récent au plus ancien et se lit page par page avec `before`
#[test]
fn list_orders_by_recency_and_pages_by_before() {
    let db = Database::open_in_memory().unwrap();
    for (id, now) in [("a", 1), ("b", 2), ("c", 3)] {
        save(&db, id, id, now);
    }
    let first = db.read(|tx| tx.list_ai_conversations(2, None)).unwrap();
    let ids: Vec<&str> = first.iter().map(|m| m.id.as_str()).collect();
    assert_eq!(ids, ["c", "b"]);
    let rest = db
        .read(|tx| tx.list_ai_conversations(2, Some("b")))
        .unwrap();
    assert_eq!(rest.len(), 1);
    assert_eq!(rest[0].id, "a");
    let unknown = db
        .read(|tx| tx.list_ai_conversations(2, Some("absent")))
        .unwrap();
    assert!(unknown.is_empty());
}

/// Lire une conversation inconnue échoue avec `not_found.ai_conversation`
#[test]
fn get_unknown_is_not_found() {
    let db = Database::open_in_memory().unwrap();
    let refused = db.read(|tx| tx.get_ai_conversation("absent"));
    assert_eq!(refused.unwrap_err().code, "not_found.ai_conversation");
}

/// Le payload est rendu tel qu'il a été écrit
#[test]
fn get_returns_payload_verbatim() {
    let db = Database::open_in_memory().unwrap();
    let text = "{\"messages\":[{\"text\":\"é 😀\"}]}";
    db.write(|tx| tx.save_ai_conversation("c-1", "T", 1, text, 5))
        .unwrap();
    let found = db.read(|tx| tx.get_ai_conversation("c-1")).unwrap();
    assert_eq!(found.payload, text);
}
