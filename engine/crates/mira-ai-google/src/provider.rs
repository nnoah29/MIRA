//! Implémentation de l'interface des fournisseurs d'IA pour Google Gemini.

use mira_vocabulary::{AiProvider, AiProviderDescriptor};

use crate::descriptor::descriptor;

/// Fournisseur d'IA Google Gemini (structure unitaire, sans état).
#[derive(Debug, Clone, Copy, Default)]
pub struct GoogleAiProvider;

impl AiProvider for GoogleAiProvider {
    /// Décrit Google Gemini avec sa fiche statique.
    fn descriptor(&self) -> AiProviderDescriptor {
        descriptor()
    }
}
