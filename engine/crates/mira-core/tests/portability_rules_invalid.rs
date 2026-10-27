#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Règles du fichier malformées : champ inconnu, identifiant absent ou vide.

mod support_portability;

use mira_core::portability::rules::{PortableError, from_portable};
use serde_json::json;
use support_portability::fake_refs::FakeRefs;
use support_portability::rule::{portable, with_field};

/// Un champ inconnu est refusé plutôt qu'ignoré
#[test]
fn unknown_field_is_invalid() {
    let value = with_field(portable(), "nom_inconnu", json!(1));
    let error = from_portable(value, &FakeRefs::default()).unwrap_err();
    assert_eq!(error, PortableError::Invalid);
}

/// Une règle sans identifiant est invalide
#[test]
fn missing_id_is_invalid() {
    let mut value = portable();
    value.as_object_mut().unwrap().remove("id");
    let error = from_portable(value, &FakeRefs::default()).unwrap_err();
    assert_eq!(error, PortableError::Invalid);
}

/// Une règle à identifiant vide est invalide
#[test]
fn empty_id_is_invalid() {
    let value = with_field(portable(), "id", json!(" "));
    let error = from_portable(value, &FakeRefs::default()).unwrap_err();
    assert_eq!(error, PortableError::Invalid);
}
