#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Le registre assemblé détecte les quatre providers réels.

use mira_assembly::build_registry;
use mira_assembly::mira_vocabulary::{EmailAddress, ProviderId};

/// Détecte l'identifiant du provider d'une adresse écrite en texte
fn detected(text: &str) -> String {
    let address = EmailAddress::parse(text).unwrap();
    build_registry().detect(&address).descriptor().id.0
}

/// Chaque service connu est détecté d'après son domaine
#[test]
fn known_domains_are_detected() {
    assert_eq!(detected("x@gmail.com"), "gmail");
    assert_eq!(detected("x@GoogleMail.com"), "gmail");
    assert_eq!(detected("x@hotmail.fr"), "outlook");
    assert_eq!(detected("x@ymail.com"), "yahoo");
}

/// Un domaine inconnu ou un sous-domaine retombe sur le repli générique
#[test]
fn unknown_domains_fall_back_to_generic() {
    assert_eq!(detected("x@inconnu.org"), "generic");
    assert_eq!(detected("x@foo.gmail.com"), "generic");
}

/// La liste contient les quatre providers, repli en dernier ; tous sont retrouvables par id
#[test]
fn list_contains_the_four_providers_with_generic_last() {
    let registry = build_registry();
    let ids: Vec<String> = registry.list().into_iter().map(|d| d.id.0).collect();
    assert_eq!(ids, ["gmail", "outlook", "yahoo", "generic"]);
    for id in &ids {
        assert!(registry.find(&ProviderId::new(id)).is_some(), "{id}");
    }
}
