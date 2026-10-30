#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Dépôt des connexions d'IA : mise à jour sélective, liste de modèles gardée et migration 16.

mod support_ai_connections;

use mira_core::storage::{Database, schema_version};
use mira_vocabulary::{AiConnectionId, AiModel, AiModelList, Timestamp};
use support_ai_connections::{code_of, connection, insert, listed_ids};

/// La mise à jour change libellé, adresse et modèle sans toucher consentement, défaut ni modèles
#[test]
fn update_does_not_touch_consent_or_default() {
    let db = Database::open_in_memory().unwrap();
    insert(&db, &connection("c-1", "A")).unwrap();
    let id = AiConnectionId::from_raw("c-1");
    let list = AiModelList {
        models: vec![],
        fetched_at: Timestamp::from_millis(9),
    };
    db.write(|tx| {
        tx.set_ai_consent(&id, 50)?;
        tx.set_default_ai_connection(&id)?;
        tx.save_ai_models(&id, &list)
    })
    .unwrap();
    db.write(|tx| tx.update_ai_connection(&id, "Renommée", None, Some("modele-b"), 99))
        .unwrap();
    let found = db.read(|tx| tx.require_ai_connection(&id)).unwrap();
    assert_eq!(found.label, "Renommée");
    assert_eq!(found.base_url, None);
    assert_eq!(found.default_model.as_deref(), Some("modele-b"));
    assert_eq!(found.consent_at, Some(Timestamp::from_millis(50)));
    assert!(found.is_default);
    assert_eq!(found.models, Some(list));
    let unknown = AiConnectionId::from_raw("absent");
    let refused = db.write(|tx| tx.update_ai_connection(&unknown, "X", None, None, 1));
    assert_eq!(code_of(refused), "not_found.ai_connection");
}

/// La migration 16 s'applique à un profil existant en version 15 et garde ses données
#[test]
fn migration_16_applies_on_existing_profile() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mira.sqlite3");
    drop(Database::open(&path).unwrap());
    let raw = rusqlite::Connection::open(&path).unwrap();
    raw.execute_batch(
        "DROP TABLE ai_journal; DROP TABLE ai_connections; PRAGMA user_version = 15;",
    )
    .unwrap();
    drop(raw);
    let db = Database::open(&path).unwrap();
    assert_eq!(schema_version(), 19);
    let version = db.read(|tx| Ok(tx.schema_version_on_disk())).unwrap();
    assert_eq!(version, 19);
    insert(&db, &connection("c-1", "A")).unwrap();
    assert!(
        db.read(|tx| tx.table_names())
            .unwrap()
            .contains(&"ai_connections".to_string())
    );
    assert_eq!(listed_ids(&db), ["c-1"]);
}

/// La liste de modèles gardée est relue avec sa date de relevé
#[test]
fn models_cache_roundtrip_with_fetched_at() {
    let db = Database::open_in_memory().unwrap();
    insert(&db, &connection("c-1", "A")).unwrap();
    let id = AiConnectionId::from_raw("c-1");
    let list = AiModelList {
        models: vec![
            AiModel {
                id: "m-1".to_string(),
                created_at: Some(7),
            },
            AiModel {
                id: "m-2".to_string(),
                created_at: None,
            },
        ],
        fetched_at: Timestamp::from_millis(4242),
    };
    db.write(|tx| tx.save_ai_models(&id, &list)).unwrap();
    let found = db.read(|tx| tx.require_ai_connection(&id)).unwrap();
    assert_eq!(found.models, Some(list));
}
