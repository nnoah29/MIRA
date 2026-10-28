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

/// Un identifiant trop long est invalide
#[test]
fn overlong_id_is_invalid() {
    let value = with_field(portable(), "id", json!("x".repeat(65)));
    let error = from_portable(value, &FakeRefs::default()).unwrap_err();
    assert_eq!(error, PortableError::Invalid);
}

/// Un identifiant avec retour à la ligne ou caractère hors alphabet est invalide
#[test]
fn id_outside_the_engine_alphabet_is_invalid() {
    for id in ["a\nb", "règle", "a b", "../x"] {
        let value = with_field(portable(), "id", json!(id));
        let error = from_portable(value, &FakeRefs::default()).unwrap_err();
        assert_eq!(error, PortableError::Invalid, "{id:?}");
    }
}

/// Une règle d'un format de schéma futur est refusée
#[test]
fn future_schema_version_is_invalid() {
    let value = with_field(portable(), "schema_version", json!(99));
    let error = from_portable(value, &FakeRefs::default()).unwrap_err();
    assert_eq!(error, PortableError::Invalid);
}
