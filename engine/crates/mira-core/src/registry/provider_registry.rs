//! Catalogue ordonné des providers connus, avec un repli pour les adresses inconnues.

use std::sync::Arc;

use mira_vocabulary::{EmailAddress, Provider, ProviderDescriptor, ProviderId};

/// Providers enregistrés (l'ordre d'enregistrement fait foi) et provider de repli.
#[derive(Debug, Clone)]
pub struct ProviderRegistry {
    providers: Vec<Arc<dyn Provider>>,
    fallback: Arc<dyn Provider>,
}

impl ProviderRegistry {
    /// Crée un registre vide qui répond par ce repli quand aucun provider ne reconnaît l'adresse.
    pub fn new(fallback: Arc<dyn Provider>) -> Self {
        Self {
            providers: Vec::new(),
            fallback,
        }
    }

    /// Ajoute un provider après ceux déjà enregistrés.
    pub fn register(&mut self, provider: Arc<dyn Provider>) {
        self.providers.push(provider);
    }

    /// Retrouve un provider par son identifiant, le repli compris.
    pub fn find(&self, id: &ProviderId) -> Option<Arc<dyn Provider>> {
        self.providers
            .iter()
            .chain(std::iter::once(&self.fallback))
            .find(|provider| provider.descriptor().id == *id)
            .cloned()
    }

    /// Choisit le premier provider enregistré qui gère l'adresse, sinon le repli (6.6).
    pub fn detect(&self, address: &EmailAddress) -> Arc<dyn Provider> {
        self.providers
            .iter()
            .find(|provider| provider.handles_address(address))
            .unwrap_or(&self.fallback)
            .clone()
    }

    /// Décrit tous les providers dans l'ordre d'enregistrement, le repli en dernier.
    pub fn list(&self) -> Vec<ProviderDescriptor> {
        self.providers
            .iter()
            .chain(std::iter::once(&self.fallback))
            .map(|provider| provider.descriptor())
            .collect()
    }
}
