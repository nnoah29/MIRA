//! Implémentation de l'interface des fournisseurs d'IA pour OpenAI.

use mira_vocabulary::{AiProvider, AiProviderDescriptor};

use crate::descriptor::descriptor;

/// Fournisseur d'IA OpenAI (structure unitaire, sans état).
#[derive(Debug, Clone, Copy, Default)]
pub struct OpenAiAiProvider;

impl AiProvider for OpenAiAiProvider {
    /// Décrit OpenAI avec sa fiche statique.
    fn descriptor(&self) -> AiProviderDescriptor {
        descriptor()
    }
}
