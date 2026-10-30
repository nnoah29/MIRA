//! Implémentation de l'interface des fournisseurs d'IA pour Groq.

use mira_vocabulary::{AiProvider, AiProviderDescriptor};

use crate::descriptor::descriptor;

/// Fournisseur d'IA Groq (structure unitaire, sans état).
#[derive(Debug, Clone, Copy, Default)]
pub struct GroqAiProvider;

impl AiProvider for GroqAiProvider {
    /// Décrit Groq avec sa fiche statique.
    fn descriptor(&self) -> AiProviderDescriptor {
        descriptor()
    }
}
