//! Implémentation de l'interface des fournisseurs d'IA pour Compatible OpenAI.

use mira_vocabulary::{AiProvider, AiProviderDescriptor};

use crate::descriptor::descriptor;

/// Fournisseur d'IA Compatible OpenAI (structure unitaire, sans état).
#[derive(Debug, Clone, Copy, Default)]
pub struct GenericAiProvider;

impl AiProvider for GenericAiProvider {
    /// Décrit Compatible OpenAI avec sa fiche statique.
    fn descriptor(&self) -> AiProviderDescriptor {
        descriptor()
    }
}
