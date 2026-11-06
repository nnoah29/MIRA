//! Version du protocole de trames (distincte de la version du contrat d'API).

use serde::{Deserialize, Serialize};

/// Version du protocole : la majeure casse la compatibilité, la mineure ajoute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub major: u16,
    pub minor: u16,
}

/// Version du protocole implémentée par ce crate.
pub const PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion { major: 1, minor: 0 };
