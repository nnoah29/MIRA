//! Profil d'avant la confidentialité : base ramenée à la version 16 puis rouverte par le moteur.
#![allow(dead_code)]

use std::path::Path;

use mira_core::storage::Database;

/// Crée un profil, le ramène à la version 16 avec ces connexions (identifiant, consentement), le rouvre
pub fn reopened_from_version_16(path: &Path, rows: &[(&str, Option<i64>)]) -> Database {
    drop(Database::open(path).unwrap());
    let raw = rusqlite::Connection::open(path).unwrap();
    raw.execute_batch(
        "DROP TABLE ai_journal;
         ALTER TABLE ai_connections DROP COLUMN allowed_kinds;
         ALTER TABLE ai_connections DROP COLUMN masking_json;
         PRAGMA user_version = 16;",
    )
    .unwrap();
    for (id, consent) in rows {
        raw.execute(
            "INSERT INTO ai_connections (id, provider_id, label, is_default, consent_at, created_at, updated_at) \
             VALUES (?1, 'test-provider', ?1, 0, ?2, 1, 1)",
            rusqlite::params![id, consent],
        )
        .unwrap();
    }
    drop(raw);
    Database::open(path).unwrap()
}
