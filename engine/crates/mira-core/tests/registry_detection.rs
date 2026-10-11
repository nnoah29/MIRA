#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Détection du provider par adresse : ordre d'enregistrement, repli (6.6).

#[allow(dead_code)]
mod provider_fakes;

use std::sync::Arc;

use mira_core::registry::ProviderRegistry;
use mira_vocabulary::ProviderId;
use provider_fakes::registries::{detected_id, standard_registry};
use provider_fakes::{FakeFoldersProvider, FakeLabelsProvider};

/// Chaque faux provider est détecté d'après le domaine de l'adresse
#[test]
fn detect_picks_the_provider_handling_the_domain() {
    let registry = standard_registry();
    assert_eq!(detected_id(&registry, "a@labels.test"), "fake_labels");
    assert_eq!(detected_id(&registry, "a@FOLDERS.test"), "fake_folders");
}

/// Une adresse inconnue, ou un sous-domaine, retombe sur le repli
#[test]
fn detect_falls_back_when_nobody_handles_the_address() {
    let registry = standard_registry();
    assert_eq!(detected_id(&registry, "a@inconnu.example"), "fake_fallback");
    assert_eq!(detected_id(&registry, "a@sub.labels.test"), "fake_fallback");
}

/// Quand deux providers gèrent le même domaine, le premier enregistré l'emporte
#[test]
fn registration_order_decides_between_two_matches() {
    let mut registry =
        ProviderRegistry::new(Arc::new(FakeFoldersProvider::new("fake_fallback", "")));
    registry.register(Arc::new(FakeFoldersProvider::new("first", "shared.test")));
    registry.register(Arc::new(FakeFoldersProvider::new("second", "shared.test")));
    assert_eq!(detected_id(&registry, "a@shared.test"), "first");
}

/// La liste suit l'ordre d'enregistrement et place le repli en dernier
#[test]
fn list_follows_registration_order_with_fallback_last() {
    let ids: Vec<String> = standard_registry()
        .list()
        .into_iter()
        .map(|descriptor| descriptor.id.as_str().to_string())
        .collect();
    assert_eq!(ids, ["fake_labels", "fake_folders", "fake_fallback"]);
}

/// Un registre sans provider enregistré répond toujours par le repli
#[test]
fn empty_registry_always_answers_with_the_fallback() {
    let registry = ProviderRegistry::new(Arc::new(FakeLabelsProvider));
    assert_eq!(detected_id(&registry, "a@labels.test"), "fake_labels");
    assert_eq!(detected_id(&registry, "a@ailleurs.test"), "fake_labels");
    assert_eq!(registry.list().len(), 1);
}

/// `find` retrouve un provider enregistré, le repli, et rien d'autre
#[test]
fn find_looks_up_registered_providers_and_the_fallback() {
    let registry = standard_registry();
    for known in ["fake_labels", "fake_folders", "fake_fallback"] {
        let found = registry.find(&ProviderId::new(known)).unwrap();
        assert_eq!(found.descriptor().id.as_str(), known);
    }
    assert!(registry.find(&ProviderId::new("absent")).is_none());
}
