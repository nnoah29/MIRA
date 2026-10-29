//! Implémentation de l'interface des fournisseurs d'IA pour Anthropic.

use mira_vocabulary::{AiProvider, AiProviderDescriptor};

use crate::descriptor::descriptor;

/// Fournisseur d'IA Anthropic (structure unitaire, sans état).
#[derive(Debug, Clone, Copy, Default)]
pub struct AnthropicAiProvider;

impl AiProvider for AnthropicAiProvider {
    /// Décrit Anthropic avec sa fiche statique.
    fn descriptor(&self) -> AiProviderDescriptor {
        descriptor()
    }
}
