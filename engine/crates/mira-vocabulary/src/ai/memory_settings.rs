//! Réglages de la mémoire de l'IA : activation, lieu du calcul des embeddings et état affiché à l'utilisateur.

use serde::{Deserialize, Serialize};

use crate::ids::kinds::AiConnectionId;

/// Où les embeddings sont calculés : sur l'appareil, ou par le service d'une connexion d'IA choisie.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MemoryEmbedder {
    #[default]
    Local,
    Provider {
        connection: AiConnectionId,
        model: String,
    },
}

/// Réglages de la mémoire ; désactivée tant que l'utilisateur ne l'a pas activée.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemorySettings {
    pub enabled: bool,
    pub embedder: MemoryEmbedder,
}

/// Ce que l'écran de la mémoire affiche : réglages, modèle local présent, nombre de souvenirs et de périmés.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryStatus {
    pub settings: MemorySettings,
    pub model_installed: bool,
    pub count: u64,
    pub stale: u64,
}
