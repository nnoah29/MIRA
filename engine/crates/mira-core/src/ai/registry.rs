//! Catalogue ordonné des fournisseurs d'IA connus, rempli par l'assemblage.

use std::sync::Arc;

use mira_vocabulary::{AiProvider, AiProviderDescriptor, AiProviderId};

/// Fournisseurs d'IA enregistrés ; l'ordre d'enregistrement fait foi.
#[derive(Debug, Clone, Default)]
pub struct AiProviderRegistry {
    providers: Vec<Arc<dyn AiProvider>>,
}

impl AiProviderRegistry {
    /// Crée un registre vide.
    pub fn new() -> Self {
        Self::default()
    }

    /// Ajoute un fournisseur après ceux déjà enregistrés.
    pub fn register(&mut self, provider: Arc<dyn AiProvider>) {
        self.providers.push(provider);
    }

    /// Retrouve un fournisseur par son identifiant.
    pub fn find(&self, id: &AiProviderId) -> Option<Arc<dyn AiProvider>> {
        self.providers
            .iter()
            .find(|provider| provider.descriptor().id == *id)
            .cloned()
    }

    /// Décrit tous les fournisseurs dans l'ordre d'enregistrement.
    pub fn list(&self) -> Vec<AiProviderDescriptor> {
        self.providers
            .iter()
            .map(|provider| provider.descriptor())
            .collect()
    }
}
