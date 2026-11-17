#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Réglages de la mémoire de l'IA rangés dans `engine_settings` : défaut, aller-retour, valeur corrompue.

use mira_core::storage::Database;
use mira_vocabulary::{AiConnectionId, MemoryEmbedder, MemorySettings};

/// Un profil neuf a la mémoire désactivée, calcul sur l'appareil
#[test]
fn memory_settings_default_when_never_set() {
    let db = Database::open_in_memory().unwrap();
    let settings = db.read(|tx| tx.ai_memory_settings()).unwrap();
    assert_eq!(settings, MemorySettings::default());
}

/// Les deux sortes d'embedder se relisent à l'identique
#[test]
fn memory_settings_round_trip_local_and_provider() {
    let db = Database::open_in_memory().unwrap();
    for embedder in [
        MemoryEmbedder::Local,
        MemoryEmbedder::Provider {
            connection: AiConnectionId::from_raw("c-1"),
            model: "embed-1".into(),
        },
    ] {
        let settings = MemorySettings {
            enabled: true,
            embedder,
        };
        db.write(|tx| tx.set_ai_memory_settings(&settings)).unwrap();
        assert_eq!(db.read(|tx| tx.ai_memory_settings()).unwrap(), settings);
    }
}

/// Une valeur illisible redonne le défaut au lieu de bloquer l'application
#[test]
fn memory_settings_corrupted_value_reads_as_default() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mira.sqlite3");
    drop(Database::open(&path).unwrap());
    let raw = rusqlite::Connection::open(&path).unwrap();
    raw.execute(
        "INSERT INTO engine_settings (key, value) VALUES ('ai_memory', 'pas du json')",
        [],
    )
    .unwrap();
    drop(raw);
    let db = Database::open(&path).unwrap();
    assert_eq!(
        db.read(|tx| tx.ai_memory_settings()).unwrap(),
        MemorySettings::default()
    );
}
