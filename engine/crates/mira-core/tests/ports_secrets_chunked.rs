#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Secrets longs : découpés en morceaux pour les trousseaux qui plafonnent la taille (Windows).

mod support_secrets;

use mira_core::ports::{SecretStore, SecretValue};
use support_secrets::{Raw, chunked, key, reference};

/// Un jeton de 3500 caractères, accents compris, que le trousseau seul refuse, se relit à l'identique.
#[test]
fn a_token_longer_than_the_keyring_limit_roundtrips() {
    let long = "é".repeat(3500);
    let store = chunked();
    store
        .put(&reference(), &SecretValue::new(long.clone()))
        .unwrap();
    let back = store.get(&reference()).unwrap().unwrap();
    assert_eq!(back.expose(), long);
}

/// Une valeur courte reste stockée telle quelle, lisible par un trousseau sans découpage.
#[test]
fn a_short_value_is_stored_unchanged() {
    let store = chunked();
    store
        .put(&reference(), &SecretValue::new("abc".to_string()))
        .unwrap();
    let raw = store.inner().get(&Raw(key())).unwrap().unwrap();
    assert_eq!(raw.expose(), "abc");
}

/// Remplacer un long secret par un court efface les morceaux devenus inutiles.
#[test]
fn replacing_a_long_secret_by_a_short_one_removes_the_extra_parts() {
    let store = chunked();
    store
        .put(&reference(), &SecretValue::new("x".repeat(3000)))
        .unwrap();
    store
        .put(&reference(), &SecretValue::new("court".to_string()))
        .unwrap();
    assert!(
        store
            .inner()
            .get(&Raw(format!("{}#0", key())))
            .unwrap()
            .is_none()
    );
    assert_eq!(store.get(&reference()).unwrap().unwrap().expose(), "court");
}

/// Supprimer un long secret supprime aussi ses morceaux.
#[test]
fn deleting_a_long_secret_removes_every_part() {
    let store = chunked();
    store
        .put(&reference(), &SecretValue::new("y".repeat(3000)))
        .unwrap();
    store.delete(&reference()).unwrap();
    assert!(store.get(&reference()).unwrap().is_none());
    assert!(
        store
            .inner()
            .get(&Raw(format!("{}#1", key())))
            .unwrap()
            .is_none()
    );
}

/// Un morceau manquant est une erreur, jamais un secret tronqué.
#[test]
fn a_missing_part_is_an_error_not_a_truncated_secret() {
    let store = chunked();
    store
        .put(&reference(), &SecretValue::new("z".repeat(3000)))
        .unwrap();
    store.inner().delete(&Raw(format!("{}#2", key()))).unwrap();
    assert!(store.get(&reference()).is_err());
}
