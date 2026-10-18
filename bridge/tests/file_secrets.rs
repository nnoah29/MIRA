#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Trousseau chiffré (Android) : aller-retour, fichier opaque, mauvaise clé refusée.

use mira_assembly::mira_vocabulary::{AccountId, ProfileId, SecretKind, SecretRef};
use mira_assembly::ports::{SecretStore, SecretValue};
use mira_bridge::file_secrets::FileSecretStore;

fn secret_ref(kind: SecretKind) -> SecretRef {
    SecretRef {
        profile: ProfileId::from_raw("default"),
        account: AccountId::from_raw("acc1"),
        kind,
    }
}

#[test]
fn put_get_delete_round_trip_and_survive_reopening() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("secrets.bin");
    let key = [7u8; 32];
    let store = FileSecretStore::new(path.clone(), &key).unwrap();
    let password = secret_ref(SecretKind::Password);
    assert!(store.get(&password).unwrap().is_none());
    store
        .put(&password, &SecretValue::new("s3cret-pass"))
        .unwrap();
    store
        .put(
            &secret_ref(SecretKind::RefreshToken),
            &SecretValue::new("tok"),
        )
        .unwrap();
    let reopened = FileSecretStore::new(path.clone(), &key).unwrap();
    assert_eq!(
        reopened.get(&password).unwrap().unwrap().expose(),
        "s3cret-pass"
    );
    reopened.delete(&password).unwrap();
    reopened.delete(&password).unwrap();
    assert!(reopened.get(&password).unwrap().is_none());
    assert_eq!(
        reopened
            .get(&secret_ref(SecretKind::RefreshToken))
            .unwrap()
            .unwrap()
            .expose(),
        "tok"
    );
}

#[test]
fn the_file_never_contains_the_secret_and_another_key_cannot_read_it() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("secrets.bin");
    let store = FileSecretStore::new(path.clone(), &[1u8; 32]).unwrap();
    store
        .put(
            &secret_ref(SecretKind::Password),
            &SecretValue::new("s3cret-pass"),
        )
        .unwrap();
    let raw = std::fs::read(&path).unwrap();
    assert!(!raw.windows(11).any(|w| w == b"s3cret-pass"));
    let intruder = FileSecretStore::new(path, &[2u8; 32]).unwrap();
    assert!(intruder.get(&secret_ref(SecretKind::Password)).is_err());
    assert!(FileSecretStore::new(dir.path().join("x"), &[0u8; 16]).is_err());
}
