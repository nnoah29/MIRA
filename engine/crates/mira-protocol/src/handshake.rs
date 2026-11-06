//! Règles de la poignée de main : version du protocole d'abord, puis compatibilité du contrat.

use mira_vocabulary::ContractVersion;
use serde::{Deserialize, Serialize};

use crate::{PROTOCOL_VERSION, ProtocolVersion};

/// Raison fixe d'un refus de `hello` ; jamais de texte libre, pour ne rien révéler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HelloRefusal {
    ProtocolVersion,
    ContractVersion,
    Auth,
    Malformed,
}

/// Vérifie les versions annoncées par le client ; le jeton se vérifie ailleurs.
pub fn check_hello(
    protocol: ProtocolVersion,
    contract: ContractVersion,
    server_contract: ContractVersion,
) -> Result<(), HelloRefusal> {
    if protocol.major != PROTOCOL_VERSION.major {
        return Err(HelloRefusal::ProtocolVersion);
    }
    if !server_contract.is_compatible_with(contract) {
        return Err(HelloRefusal::ContractVersion);
    }
    Ok(())
}
