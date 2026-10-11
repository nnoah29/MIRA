#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Trousseau réel : seul test qui le touche, lancé à la main.

use mira_core::ports::{KeyringSecretStore, SecretStore, SecretValue};
use mira_vocabulary::{AccountId, ProfileId, SecretKind, SecretRef};

/// Aller-retour complet dans le vrai trousseau du système, avec nettoyage
#[test]
#[ignore = "manuel : utilise le vrai trousseau"]
fn keyring_roundtrip_with_cleanup() {
    let store = KeyringSecretStore;
    let key = SecretRef {
        profile: ProfileId::from_raw("manual-test"),
        account: AccountId::from_raw("manual-account"),
        kind: SecretKind::Password,
    };
    assert!(store.get(&key).unwrap().is_none());
    store.put(&key, &SecretValue::new("manual-secret")).unwrap();
    assert_eq!(store.get(&key).unwrap().unwrap().expose(), "manual-secret");
    store.delete(&key).unwrap();
    assert!(store.get(&key).unwrap().is_none());
}
