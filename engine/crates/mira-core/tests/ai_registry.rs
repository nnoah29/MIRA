#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Registre des fournisseurs d'IA : ordre d'enregistrement et recherche par identifiant.

mod ai_models_support;
mod support_api;

use std::sync::Arc;

use ai_models_support::TestProvider;
use mira_core::ai::AiProviderRegistry;
use mira_vocabulary::AiProviderId;

/// La liste suit l'ordre d'enregistrement et chaque fournisseur se retrouve par son identifiant
#[test]
fn registry_lists_in_registration_order() {
    let mut registry = AiProviderRegistry::new();
    for id in ["zeta", "alpha", "mid"] {
        registry.register(Arc::new(TestProvider::new(id)));
    }
    let listed: Vec<String> = registry.list().into_iter().map(|item| item.id.0).collect();
    assert_eq!(listed, ["zeta", "alpha", "mid"]);
    let found = registry.find(&AiProviderId("alpha".to_string())).unwrap();
    assert_eq!(found.descriptor().id.0, "alpha");
}

/// Un identifiant inconnu ne rend rien, et un registre vide ne liste rien
#[test]
fn registry_find_unknown_is_none() {
    let registry = AiProviderRegistry::default();
    assert!(registry.list().is_empty());
    assert!(registry.find(&AiProviderId("nope".to_string())).is_none());
    let mut filled = registry.clone();
    filled.register(Arc::new(TestProvider::new("known")));
    assert!(filled.find(&AiProviderId("nope".to_string())).is_none());
    assert!(format!("{filled:?}").contains("AiProviderRegistry"));
}
