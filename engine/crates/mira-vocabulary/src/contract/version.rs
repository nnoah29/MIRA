//! Version majeure/mineure du contrat d'API (12.9, invariant 33).

use serde::{Deserialize, Serialize};

/// Version du contrat : la majeure casse la compatibilité, la mineure ajoute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ContractVersion {
    pub major: u32,
    pub minor: u32,
}

/// Version du contrat implémentée par ce moteur.
pub const CONTRACT_VERSION: ContractVersion = ContractVersion { major: 1, minor: 0 };

impl ContractVersion {
    /// Dit si un client écrit pour `client` peut parler à ce moteur.
    pub fn is_compatible_with(&self, client: ContractVersion) -> bool {
        self.major == client.major && client.minor <= self.minor
    }
}
