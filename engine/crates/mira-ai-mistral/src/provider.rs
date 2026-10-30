//! Implémentation de l'interface des fournisseurs d'IA pour Mistral AI.

use mira_vocabulary::{AiProvider, AiProviderDescriptor};

use crate::descriptor::descriptor;

/// Fournisseur d'IA Mistral AI (structure unitaire, sans état).
#[derive(Debug, Clone, Copy, Default)]
pub struct MistralAiProvider;

impl AiProvider for MistralAiProvider {
    /// Décrit Mistral AI avec sa fiche statique.
    fn descriptor(&self) -> AiProviderDescriptor {
        descriptor()
    }
}
