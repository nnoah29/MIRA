//! Implémentation de l'interface des fournisseurs d'IA pour ChatGPT.

use mira_vocabulary::{AiProvider, AiProviderDescriptor};

use crate::descriptor::descriptor;

/// Fournisseur d'IA ChatGPT (structure unitaire, sans état).
#[derive(Debug, Clone, Copy, Default)]
pub struct ChatGptAiProvider;

impl AiProvider for ChatGptAiProvider {
    /// Décrit ChatGPT avec sa fiche statique.
    fn descriptor(&self) -> AiProviderDescriptor {
        descriptor()
    }
}
