//! Capacités effectives : ce que le serveur annonce, moins les exceptions déclarées par le provider (D-08).

use std::collections::BTreeSet;

use mira_vocabulary::ProviderQuirks;

use super::server_capability::ServerCapability;

/// Capacités sur lesquelles le moteur peut compter pour un compte.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EffectiveCapabilities {
    available: BTreeSet<ServerCapability>,
}

impl EffectiveCapabilities {
    /// Retire de l'annonce du serveur les extensions que le provider déclare peu fiables.
    pub fn compute(announced: &BTreeSet<ServerCapability>, quirks: &ProviderQuirks) -> Self {
        let unreliable: BTreeSet<ServerCapability> = quirks
            .unreliable_extensions
            .iter()
            .filter_map(|name| ServerCapability::from_imap_name(name))
            .collect();
        Self {
            available: announced.difference(&unreliable).copied().collect(),
        }
    }

    /// Dit si la capacité est utilisable.
    pub fn has(&self, capability: ServerCapability) -> bool {
        self.available.contains(&capability)
    }

    /// Donne l'ensemble des capacités utilisables.
    pub fn all(&self) -> &BTreeSet<ServerCapability> {
        &self.available
    }
}
