#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Invariant 15 : aucun secret n'est écrit dans la base.

mod support;

use std::path::Path;

use mira_core::ports::{MemorySecretStore, SecretStore, SecretValue};
use mira_core::storage::Database;
use mira_vocabulary::{AccountId, EventKind, ProfileId, SecretKind, SecretRef, Timestamp};
use support::accounts::{sample_account, sample_identity};
use support::intentions::{sample_action, sample_draft};

/// Dit si un fichier du dossier (base, journal WAL, mémoire partagée) contient la valeur
fn any_file_contains(dir: &Path, needle: &str) -> bool {
    std::fs::read_dir(dir).unwrap().any(|entry| {
        let bytes = std::fs::read(entry.unwrap().path()).unwrap();
        bytes
            .windows(needle.len())
            .any(|window| window == needle.as_bytes())
    })
}

/// Invariant 15 : après un scénario complet, la base ne contient aucune valeur secrète
#[test]
fn invariant_15_no_secret_in_database() {
    let dir = tempfile::tempdir().unwrap();
    let secret = "S3cr3t-passw0rd-VALUE-9f2c";
    let store = MemorySecretStore::new();
    let key = SecretRef {
        profile: ProfileId::from_raw("default"),
        account: AccountId::from_raw("acc-1"),
        kind: SecretKind::Password,
    };
    store.put(&key, &SecretValue::new(secret)).unwrap();
    let db = Database::open(&dir.path().join("mira.sqlite3")).unwrap();
    db.write(|tx| {
        tx.insert_account(&sample_account("acc-1"))?;
        tx.insert_identity(&sample_identity("main", "acc-1", true))?;
        tx.insert_draft(&sample_draft("d-1", "acc-1", 5))?;
        tx.enqueue_action(&sample_action("a-1", "acc-1", "k1", 0))?;
        tx.append_event(Timestamp::from_millis(1), &EventKind::EngineStarted)
    })
    .unwrap();
    assert_eq!(store.get(&key).unwrap().unwrap().expose(), secret);
    assert!(!any_file_contains(dir.path(), secret));
    assert!(any_file_contains(dir.path(), "alice@example.org"));
}
