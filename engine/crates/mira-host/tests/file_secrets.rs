#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Trousseau chiffré (Android) : aller-retour, fichier opaque, mauvaise clé refusée.

use mira_assembly::mira_vocabulary::{
    AccountId, AiConnectionId, AiSecretRef, ProfileId, SecretKind, SecretRef,
};
use mira_assembly::ports::{SecretStore, SecretValue};
use mira_host::file_secrets::FileSecretStore;

/// Référence de secret du compte `acc1` pour ce genre de secret.
fn secret_ref(kind: SecretKind) -> SecretRef {
    SecretRef {
        profile: ProfileId::from_raw("default"),
        account: AccountId::from_raw("acc1"),
        kind,
    }
}

/// Les secrets écrits survivent à la réouverture du fichier et la suppression est idempotente.
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
    let token_ref = secret_ref(SecretKind::RefreshToken);
    store.put(&token_ref, &SecretValue::new("tok")).unwrap();
    let reopened = FileSecretStore::new(path.clone(), &key).unwrap();
    assert_eq!(
        reopened.get(&password).unwrap().unwrap().expose(),
        "s3cret-pass"
    );
    reopened.delete(&password).unwrap();
    reopened.delete(&password).unwrap();
    assert!(reopened.get(&password).unwrap().is_none());
    let token = reopened.get(&token_ref).unwrap().unwrap();
    assert_eq!(token.expose(), "tok");
}

/// Le fichier ne contient jamais le secret en clair et une autre clé ne le lit pas.
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

/// Un secret de connexion d'IA fait l'aller-retour comme un secret de compte.
#[test]
fn file_store_roundtrips_an_ai_secret() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("secrets.bin");
    let key = [9u8; 32];
    let ai = AiSecretRef {
        profile: ProfileId::from_raw("default"),
        connection: AiConnectionId::from_raw("conn1"),
    };
    let store = FileSecretStore::new(path.clone(), &key).unwrap();
    store
        .put(&secret_ref(SecretKind::Password), &SecretValue::new("pw"))
        .unwrap();
    store.put(&ai, &SecretValue::new("sk-ai-key")).unwrap();
    let reopened = FileSecretStore::new(path.clone(), &key).unwrap();
    assert_eq!(reopened.get(&ai).unwrap().unwrap().expose(), "sk-ai-key");
    let password = reopened.get(&secret_ref(SecretKind::Password));
    assert_eq!(password.unwrap().unwrap().expose(), "pw");
    let raw = std::fs::read(&path).unwrap();
    assert!(!raw.windows(9).any(|w| w == b"sk-ai-key"));
    reopened.delete(&ai).unwrap();
    reopened.delete(&ai).unwrap();
    assert!(reopened.get(&ai).unwrap().is_none());
}
