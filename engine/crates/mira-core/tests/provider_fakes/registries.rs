//! Registres prêts à l'emploi pour les tests du cœur.

use std::sync::Arc;

use mira_core::registry::ProviderRegistry;
use mira_vocabulary::EmailAddress;

use super::{FakeFoldersProvider, FakeLabelsProvider};

/// Construit un registre avec un repli, un faux provider à libellés et un à dossiers
pub fn standard_registry() -> ProviderRegistry {
    let mut registry =
        ProviderRegistry::new(Arc::new(FakeFoldersProvider::new("fake_fallback", "")));
    registry.register(Arc::new(FakeLabelsProvider));
    registry.register(Arc::new(FakeFoldersProvider::new(
        "fake_folders",
        "folders.test",
    )));
    registry
}

/// Détecte l'identifiant du provider d'une adresse écrite en texte
#[allow(clippy::unwrap_used)]
pub fn detected_id(registry: &ProviderRegistry, text: &str) -> String {
    let address = EmailAddress::parse(text).unwrap();
    registry
        .detect(&address)
        .descriptor()
        .id
        .as_str()
        .to_string()
}
