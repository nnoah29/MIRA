#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Résolution du provider d'un compte : provider introuvable (7.8).

#[allow(dead_code)]
mod provider_fakes;

use std::sync::Arc;

use mira_core::registry::{ProviderRegistry, resolve_account_provider};
use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{ErrorCategory, ProviderId};
use provider_fakes::{FakeFoldersProvider, FakeLabelsProvider};

/// Construit un registre dont le repli est un faux provider à dossiers
fn registry() -> ProviderRegistry {
    let mut registry =
        ProviderRegistry::new(Arc::new(FakeFoldersProvider::new("fake_fallback", "")));
    registry.register(Arc::new(FakeLabelsProvider));
    registry
}

/// Un identifiant connu se résout, repli compris
#[test]
fn known_provider_ids_resolve() {
    let registry = registry();
    for known in ["fake_labels", "fake_fallback"] {
        let provider = resolve_account_provider(&registry, &ProviderId::new(known)).unwrap();
        assert_eq!(provider.descriptor().id.as_str(), known);
    }
}

/// Un identifiant retiré du catalogue donne `not_found.provider`, sans secret
#[test]
fn unknown_provider_id_gives_not_found_provider() {
    let error = resolve_account_provider(&registry(), &ProviderId::new("retire")).unwrap_err();
    assert_eq!(error.category, ErrorCategory::NotFound);
    assert_eq!(error.code, not_found::PROVIDER.0);
    assert_eq!(error.code, "not_found.provider");
    assert!(error.message.contains("retire"));
    assert!(!error.retryable);
}
