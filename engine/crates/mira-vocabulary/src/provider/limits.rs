//! Plafonds de politesse envers un service.

use serde::{Deserialize, Serialize};

/// Limites que le moteur respecte pour ne pas se faire bloquer par le service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProviderLimits {
    pub max_connections: u32,
    pub min_poll_interval_secs: u32,
}

impl Default for ProviderLimits {
    /// Donne des limites prudentes : 3 connexions, une interrogation toutes les 300 secondes.
    fn default() -> Self {
        Self {
            max_connections: 3,
            min_poll_interval_secs: 300,
        }
    }
}
