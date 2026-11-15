//! Fournisseur de test qui rend une fiche donnée, sans nom de fournisseur réel.

use mira_vocabulary::{AiProvider, AiProviderDescriptor};

/// Fournisseur de test qui rend une fiche donnée
#[derive(Debug)]
pub struct LocalTestProvider(pub AiProviderDescriptor);

impl AiProvider for LocalTestProvider {
    /// Rend la fiche portée par le fournisseur
    fn descriptor(&self) -> AiProviderDescriptor {
        self.0.clone()
    }
}
