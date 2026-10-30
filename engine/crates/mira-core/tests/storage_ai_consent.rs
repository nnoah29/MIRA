#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Dépôt des connexions d'IA : consentement retiré, liste de modèles illisible relue comme absente.

#[allow(dead_code)]
mod support_ai_connections;

use mira_core::storage::Database;
use mira_vocabulary::{AiConnectionId, AiModelList, Timestamp};
use support_ai_connections::{code_of, connection, insert};

/// Retirer le consentement remet la date à zéro ; une connexion inconnue est refusée
#[test]
fn clear_consent_resets_the_date() {
    let db = Database::open_in_memory().unwrap();
    insert(&db, &connection("c-1", "A")).unwrap();
    let id = AiConnectionId::from_raw("c-1");
    db.write(|tx| tx.set_ai_consent(&id, 50)).unwrap();
    let given = db.read(|tx| tx.require_ai_connection(&id)).unwrap();
    assert_eq!(given.consent_at, Some(Timestamp::from_millis(50)));
    db.write(|tx| tx.clear_ai_consent(&id)).unwrap();
    let cleared = db.read(|tx| tx.require_ai_connection(&id)).unwrap();
    assert_eq!(cleared.consent_at, None);
    let unknown = AiConnectionId::from_raw("absente");
    assert_eq!(
        code_of(db.write(|tx| tx.clear_ai_consent(&unknown))),
        "not_found.ai_connection"
    );
}

/// Une liste de modèles gardée illisible se relit comme absente, sans faire échouer les lectures
#[test]
fn unreadable_models_cache_reads_as_absent() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mira.sqlite3");
    let db = Database::open(&path).unwrap();
    insert(&db, &connection("c-1", "A")).unwrap();
    insert(&db, &connection("c-2", "B")).unwrap();
    let raw = rusqlite::Connection::open(&path).unwrap();
    raw.execute(
        "UPDATE ai_connections SET models_json = '{pas du json', models_fetched_at = 7 WHERE id = 'c-1'",
        [],
    )
    .unwrap();
    let listed = db.read(|tx| tx.list_ai_connections()).unwrap();
    assert_eq!(listed.len(), 2);
    assert!(listed.iter().all(|item| item.models.is_none()));
    let single = db
        .read(|tx| tx.require_ai_connection(&AiConnectionId::from_raw("c-1")))
        .unwrap();
    assert_eq!(single.label, "A");
}

/// Oublier les modèles vide la liste gardée ; une connexion inconnue est refusée
#[test]
fn clear_models_forgets_the_cached_list() {
    let db = Database::open_in_memory().unwrap();
    insert(&db, &connection("c-1", "A")).unwrap();
    let id = AiConnectionId::from_raw("c-1");
    let list = AiModelList {
        models: vec![],
        fetched_at: Timestamp::from_millis(9),
    };
    db.write(|tx| tx.save_ai_models(&id, &list)).unwrap();
    db.write(|tx| tx.clear_ai_models(&id)).unwrap();
    assert_eq!(
        db.read(|tx| tx.require_ai_connection(&id)).unwrap().models,
        None
    );
    let unknown = AiConnectionId::from_raw("absente");
    assert_eq!(
        code_of(db.write(|tx| tx.clear_ai_models(&unknown))),
        "not_found.ai_connection"
    );
}
