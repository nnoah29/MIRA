//! Réponses du contrat : une variante par forme de données rendue.

use mira_vocabulary::{Account, ContractVersion, Event, ProfileId, ProviderDescriptor};
use serde::{Deserialize, Serialize};

use crate::operations::OperationInfo;

/// Version du contrat et capacités du moteur (12.4, Moteur).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractInfo {
    pub version: ContractVersion,
    pub engine_capabilities: Vec<String>,
}

/// État global du moteur : profil, emplacement des données, comptes et actions en attente.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EngineStatus {
    pub profile: ProfileId,
    pub data_directory: String,
    pub accounts: Vec<AccountStatus>,
}

/// Compte et nombre de ses actions encore en attente.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountStatus {
    pub account: Account,
    pub pending_actions: u64,
}

/// Toute réponse du moteur.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum Response {
    Done,
    ContractInfo(ContractInfo),
    Status(EngineStatus),
    Providers(Vec<ProviderDescriptor>),
    Provider(ProviderDescriptor),
    Operation(OperationInfo),
    Events(Vec<Event>),
}
