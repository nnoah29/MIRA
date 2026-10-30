#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Dépôt des connexions d'IA : aller-retour, tri, défaut unique, consentement, modèles.

mod support_ai_connections;

use mira_core::storage::Database;
use mira_vocabulary::{AiConnectionId, Timestamp};
use support_ai_connections::{code_of, connection, insert, listed_ids};

/// Une connexion insérée est relue à l'identique, `has_key` restant faux
#[test]
fn insert_then_get_roundtrip() {
    let db = Database::open_in_memory().unwrap();
    let item = connection("c-1", "Perso");
    insert(&db, &item).unwrap();
    let found = db.read(|tx| tx.get_ai_connection(&item.id)).unwrap();
    assert_eq!(found, Some(item.clone()));
    let absent = AiConnectionId::from_raw("absent");
    assert!(
        db.read(|tx| tx.get_ai_connection(&absent))
            .unwrap()
            .is_none()
    );
}

/// Les connexions sont rendues par libellé sans tenir compte de la casse, puis par identifiant
#[test]
fn list_is_sorted_by_label() {
    let db = Database::open_in_memory().unwrap();
    insert(&db, &connection("c-3", "zèbre")).unwrap();
    insert(&db, &connection("c-2", "Alpha")).unwrap();
    insert(&db, &connection("c-1", "alpha")).unwrap();
    assert_eq!(listed_ids(&db), ["c-1", "c-2", "c-3"]);
}

/// Au plus une connexion est par défaut, même après plusieurs changements
#[test]
fn only_one_default_after_set_default() {
    let db = Database::open_in_memory().unwrap();
    insert(&db, &connection("c-1", "A")).unwrap();
    insert(&db, &connection("c-2", "B")).unwrap();
    for name in ["c-1", "c-2", "c-1"] {
        let id = AiConnectionId::from_raw(name);
        db.write(|tx| tx.set_default_ai_connection(&id)).unwrap();
    }
    let all = db.read(|tx| tx.list_ai_connections()).unwrap();
    let defaults: Vec<&str> = all
        .iter()
        .filter(|item| item.is_default)
        .map(|item| item.id.as_str())
        .collect();
    assert_eq!(defaults, ["c-1"]);
}

/// Désigner une nouvelle connexion par défaut retire l'ancien défaut ; un inconnu est refusé
#[test]
fn set_default_switches_previous_one_off() {
    let db = Database::open_in_memory().unwrap();
    insert(&db, &connection("c-1", "A")).unwrap();
    insert(&db, &connection("c-2", "B")).unwrap();
    let (first, second) = (
        AiConnectionId::from_raw("c-1"),
        AiConnectionId::from_raw("c-2"),
    );
    db.write(|tx| tx.set_default_ai_connection(&first)).unwrap();
    db.write(|tx| tx.set_default_ai_connection(&second))
        .unwrap();
    assert!(
        !db.read(|tx| tx.require_ai_connection(&first))
            .unwrap()
            .is_default
    );
    assert!(
        db.read(|tx| tx.require_ai_connection(&second))
            .unwrap()
            .is_default
    );
    let unknown = AiConnectionId::from_raw("absent");
    let refused = db.write(|tx| tx.set_default_ai_connection(&unknown));
    assert_eq!(code_of(refused), "not_found.ai_connection");
    assert!(
        db.read(|tx| tx.require_ai_connection(&second))
            .unwrap()
            .is_default
    );
}

/// Supprimer, exiger ou marquer une connexion inconnue donne `not_found.ai_connection`
#[test]
fn delete_unknown_is_not_found() {
    let db = Database::open_in_memory().unwrap();
    let id = AiConnectionId::from_raw("absent");
    assert_eq!(
        code_of(db.write(|tx| tx.delete_ai_connection(&id))),
        "not_found.ai_connection"
    );
    assert_eq!(
        code_of(db.write(|tx| tx.set_ai_consent(&id, 5))),
        "not_found.ai_connection"
    );
    let required = db.read(|tx| tx.require_ai_connection(&id));
    assert_eq!(required.unwrap_err().code, "not_found.ai_connection");
    insert(&db, &connection("c-1", "A")).unwrap();
    let known = AiConnectionId::from_raw("c-1");
    db.write(|tx| tx.delete_ai_connection(&known)).unwrap();
    assert!(listed_ids(&db).is_empty());
}

/// Le consentement garde sa première date : le redonner ne la change pas
#[test]
fn consent_is_stored_once() {
    let db = Database::open_in_memory().unwrap();
    insert(&db, &connection("c-1", "A")).unwrap();
    let id = AiConnectionId::from_raw("c-1");
    db.write(|tx| tx.set_ai_consent(&id, 100)).unwrap();
    db.write(|tx| tx.set_ai_consent(&id, 200)).unwrap();
    let found = db.read(|tx| tx.require_ai_connection(&id)).unwrap();
    assert_eq!(found.consent_at, Some(Timestamp::from_millis(100)));
}
