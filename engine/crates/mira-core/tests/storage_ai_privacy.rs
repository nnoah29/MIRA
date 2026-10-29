#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Confidentialité d'une connexion d'IA : migration 17, types autorisés et masquage relus.

#[allow(dead_code)]
mod support_ai_connections;

use mira_core::storage::{Database, schema_version};
use mira_vocabulary::{AiConnectionId, ContentKind, MaskingMode, MaskingSettings};
use support_ai_connections::profile::reopened_from_version_16;
use support_ai_connections::{code_of, connection, insert};

/// La migration 17 laisse « mail ouvert » aux connexions qui avaient consenti
#[test]
fn migration_17_keeps_consent_connections_allowed_for_mail_message() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mira.sqlite3");
    let db = reopened_from_version_16(&path, &[("c-1", Some(50))]);
    assert!(schema_version() >= 17);
    let version = db.read(|tx| Ok(tx.schema_version_on_disk())).unwrap();
    assert_eq!(version, i64::try_from(schema_version()).unwrap());
    let found = db.read(|tx| tx.require_ai_connection(&AiConnectionId::from_raw("c-1")));
    let found = found.unwrap();
    assert_eq!(found.allowed_kinds, [ContentKind::MailMessage]);
    assert_eq!(found.masking, MaskingSettings::remote_default());
}

/// La migration 17 ne donne aucun type à une connexion sans consentement
#[test]
fn migration_17_leaves_unconsented_connections_empty() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mira.sqlite3");
    let db = reopened_from_version_16(&path, &[("c-1", None)]);
    let found = db.read(|tx| tx.require_ai_connection(&AiConnectionId::from_raw("c-1")));
    assert!(found.unwrap().allowed_kinds.is_empty());
}

/// Les types autorisés se remplacent et se relisent dans l'ordre ; une connexion inconnue est refusée
#[test]
fn set_allowed_kinds_roundtrip() {
    let db = Database::open_in_memory().unwrap();
    insert(&db, &connection("c-1", "A")).unwrap();
    let id = AiConnectionId::from_raw("c-1");
    let kinds = [ContentKind::MailList, ContentKind::MailMessage];
    db.write(|tx| tx.set_ai_allowed_kinds(&id, &kinds)).unwrap();
    let found = db.read(|tx| tx.require_ai_connection(&id)).unwrap();
    assert_eq!(found.allowed_kinds, kinds);
    db.write(|tx| tx.set_ai_allowed_kinds(&id, &[])).unwrap();
    assert!(
        db.read(|tx| tx.require_ai_connection(&id))
            .unwrap()
            .allowed_kinds
            .is_empty()
    );
    let unknown = AiConnectionId::from_raw("absent");
    let refused = db.write(|tx| tx.set_ai_allowed_kinds(&unknown, &kinds));
    assert_eq!(code_of(refused), "not_found.ai_connection");
}

/// Le réglage du masquage se remplace et se relit à l'identique
#[test]
fn set_masking_roundtrip() {
    let db = Database::open_in_memory().unwrap();
    insert(&db, &connection("c-1", "A")).unwrap();
    let id = AiConnectionId::from_raw("c-1");
    let local = MaskingSettings::local_default();
    db.write(|tx| tx.set_ai_masking(&id, &local)).unwrap();
    let found = db.read(|tx| tx.require_ai_connection(&id)).unwrap();
    assert_eq!(found.masking.mode, MaskingMode::Off);
    assert_eq!(found.masking, local);
    let unknown = AiConnectionId::from_raw("absent");
    let refused = db.write(|tx| tx.set_ai_masking(&unknown, &local));
    assert_eq!(code_of(refused), "not_found.ai_connection");
}

/// Une connexion insérée porte le masquage des connexions distantes, y compris par défaut de colonne
#[test]
fn insert_connection_defaults_masking_to_remote() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mira.sqlite3");
    let db = Database::open(&path).unwrap();
    insert(&db, &connection("c-1", "A")).unwrap();
    let raw = rusqlite::Connection::open(&path).unwrap();
    raw.execute_batch(
        "INSERT INTO ai_connections (id, provider_id, label, is_default, created_at, updated_at) \
         VALUES ('c-2', 'p', 'B', 0, 1, 1)",
    )
    .unwrap();
    for name in ["c-1", "c-2"] {
        let id = AiConnectionId::from_raw(name);
        let found = db.read(|tx| tx.require_ai_connection(&id)).unwrap();
        assert_eq!(found.masking, MaskingSettings::remote_default());
        assert!(found.allowed_kinds.is_empty());
    }
}
