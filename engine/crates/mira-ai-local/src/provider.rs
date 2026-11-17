//! Implémentation de l'interface des fournisseurs d'IA pour le modèle local.

use mira_vocabulary::{AiProvider, AiProviderDescriptor};

use crate::descriptor::descriptor;

/// Fournisseur d'IA local (structure unitaire, sans état).
#[derive(Debug, Clone, Copy, Default)]
pub struct LocalAiProvider;

impl AiProvider for LocalAiProvider {
    /// Décrit le modèle local avec sa fiche statique.
    fn descriptor(&self) -> AiProviderDescriptor {
        descriptor()
    }
}
