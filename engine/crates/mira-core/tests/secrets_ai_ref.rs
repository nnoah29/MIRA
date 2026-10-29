#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Le trousseau en mémoire range aussi les clés d'API des connexions d'IA.

use mira_core::ports::{MemorySecretStore, SecretStore, SecretValue};
use mira_vocabulary::{AiConnectionId, AiSecretRef, ProfileId};

/// Référence de clé d'API de test
fn ai_reference(connection: &str) -> AiSecretRef {
    AiSecretRef {
        profile: ProfileId::from_raw("default"),
        connection: AiConnectionId::from_raw(connection),
    }
}

/// Une clé d'API d'IA s'enregistre, se relit et se remplace sans toucher aux autres.
#[test]
fn memory_store_roundtrips_an_ai_secret() {
    let store = MemorySecretStore::new();
    let key = ai_reference("c1");
    store.put(&key, &SecretValue::new("key-one")).unwrap();
    store
        .put(&ai_reference("c2"), &SecretValue::new("key-two"))
        .unwrap();
    assert_eq!(store.get(&key).unwrap().unwrap().expose(), "key-one");
    store.put(&key, &SecretValue::new("key-new")).unwrap();
    assert_eq!(store.get(&key).unwrap().unwrap().expose(), "key-new");
    assert_eq!(store.entry_count(), 2);
}

/// Supprimer deux fois une clé d'API d'IA n'est pas une erreur et la fait disparaître.
#[test]
fn memory_store_delete_is_idempotent_for_ai_secret() {
    let store = MemorySecretStore::new();
    let key = ai_reference("c1");
    store.put(&key, &SecretValue::new("key-one")).unwrap();
    store.delete(&key).unwrap();
    store.delete(&key).unwrap();
    assert!(store.get(&key).unwrap().is_none());
}

/// Le Debug du trousseau et l'erreur d'un trousseau cassé ne citent jamais la clé d'API.
#[test]
fn ai_secret_never_leaks_in_debug_or_errors() {
    let store = MemorySecretStore::new();
    store
        .put(&ai_reference("c1"), &SecretValue::new("sk-very-secret"))
        .unwrap();
    assert!(!format!("{store:?}").contains("sk-very-secret"));
    let broken = MemorySecretStore::unavailable();
    let error = broken
        .put(&ai_reference("c1"), &SecretValue::new("sk-very-secret"))
        .unwrap_err();
    assert!(!format!("{error:?} {error}").contains("sk-very-secret"));
}
