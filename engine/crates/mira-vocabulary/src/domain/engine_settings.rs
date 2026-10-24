//! Réglages que le moteur applique lui-même : source de vérité unique, partagée avec l'application.

use serde::{Deserialize, Serialize};

/// Plus petit relevé périodique accepté, en secondes.
pub const MIN_POLL_SECS: u32 = 1;
/// Plus grand relevé périodique accepté, en secondes.
pub const MAX_POLL_SECS: u32 = 3600;

/// Réglages du moteur ; une valeur absente laisse le choix au moteur (délai du provider pour le relevé).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct EngineSettings {
    pub poll_secs: Option<u32>,
}

impl EngineSettings {
    /// Noms des réglages qui diffèrent de `other`, dans un ordre stable
    pub fn changed_keys(&self, other: &Self) -> Vec<String> {
        let mut keys = Vec::new();
        if self.poll_secs != other.poll_secs {
            keys.push("poll_secs".to_string());
        }
        keys
    }
}
