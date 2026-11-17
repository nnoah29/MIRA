//! Implémentation de l'interface des fournisseurs d'IA pour OpenRouter.

use mira_vocabulary::{AiProvider, AiProviderDescriptor};

use crate::descriptor::descriptor;

/// Fournisseur d'IA OpenRouter (structure unitaire, sans état).
#[derive(Debug, Clone, Copy, Default)]
pub struct OpenRouterAiProvider;

impl AiProvider for OpenRouterAiProvider {
    /// Décrit OpenRouter avec sa fiche statique.
    fn descriptor(&self) -> AiProviderDescriptor {
        descriptor()
    }
}
