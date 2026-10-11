#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Valeur de secret masquée et trousseau en mémoire.

use mira_core::ports::{MemorySecretStore, SecretStore, SecretValue};
use mira_vocabulary::{AccountId, ProfileId, SecretKind, SecretRef};

/// Référence de secret de test
fn reference(kind: SecretKind) -> SecretRef {
    SecretRef {
        profile: ProfileId::from_raw("default"),
        account: AccountId::from_raw("acc-1"),
        kind,
    }
}

/// Un secret n'apparaît ni avec Debug ni avec Display
#[test]
fn secret_value_is_masked_in_debug_and_display() {
    let secret = SecretValue::new("hunter2-very-secret");
    assert_eq!(format!("{secret:?}"), "***");
    assert_eq!(format!("{secret}"), "***");
    assert_eq!(secret.expose(), "hunter2-very-secret");
}

/// Un trousseau mémoire rempli ne laisse pas fuiter ses valeurs dans son Debug
#[test]
fn memory_store_debug_does_not_leak_values() {
    let store = MemorySecretStore::new();
    let value = SecretValue::new("tok-abcdef-123");
    store
        .put(&reference(SecretKind::AccessToken), &value)
        .unwrap();
    assert!(!format!("{store:?}").contains("tok-abcdef-123"));
}

/// Ce qu'on enregistre se relit, se remplace et se supprime
#[test]
fn memory_store_roundtrip_replace_and_delete() {
    let store = MemorySecretStore::new();
    let key = reference(SecretKind::Password);
    assert!(store.get(&key).unwrap().is_none());
    store.put(&key, &SecretValue::new("one")).unwrap();
    store.put(&key, &SecretValue::new("two")).unwrap();
    assert_eq!(store.get(&key).unwrap().unwrap().expose(), "two");
    store.delete(&key).unwrap();
    store.delete(&key).unwrap();
    assert!(store.get(&key).unwrap().is_none());
}

/// Les secrets de natures différentes d'un même compte sont distincts
#[test]
fn memory_store_separates_secret_kinds() {
    let store = MemorySecretStore::new();
    store
        .put(
            &reference(SecretKind::AccessToken),
            &SecretValue::new("access"),
        )
        .unwrap();
    let refresh = store.get(&reference(SecretKind::RefreshToken)).unwrap();
    assert!(refresh.is_none());
}

/// Un trousseau indisponible échoue avec le code du contrat et sans citer de valeur
#[test]
fn unavailable_store_reports_keyring_unavailable() {
    let store = MemorySecretStore::unavailable();
    let key = reference(SecretKind::Password);
    let error = store
        .put(&key, &SecretValue::new("never-shown"))
        .unwrap_err();
    assert_eq!(error.code, "secrets.keyring_unavailable");
    assert!(!error.message.contains("never-shown"));
    assert!(store.get(&key).is_err());
    assert!(store.delete(&key).is_err());
}
