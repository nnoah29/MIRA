//! Interface qu'implémente chaque fournisseur d'IA : pure, sans entrée-sortie.

use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use super::descriptor::AiProviderDescriptor;

/// Clé du catalogue des fournisseurs d'IA (`anthropic`, `openai`…).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AiProviderId(pub String);

/// Un service d'IA connu du moteur ; seul `mira-assembly` les connaît tous.
pub trait AiProvider: Send + Sync + Debug {
    /// Décrit le fournisseur (données pures).
    fn descriptor(&self) -> AiProviderDescriptor;
}
