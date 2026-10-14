//! Rendu humain du contrat et de l'état du moteur.

use mira_assembly::api::{ContractInfo, EngineStatus};

use super::accounts::accounts_table;

/// Version du contrat et capacités annoncées.
pub fn contract_text(info: &ContractInfo) -> String {
    format!(
        "Contrat : {}.{}\nCapacités : {}",
        info.version.major,
        info.version.minor,
        info.engine_capabilities.join(", ")
    )
}

/// Profil, dossier de données, comptes et actions en attente.
pub fn status_text(status: &EngineStatus) -> String {
    let accounts: Vec<_> = status
        .accounts
        .iter()
        .map(|entry| entry.account.clone())
        .collect();
    let pending: u64 = status
        .accounts
        .iter()
        .map(|entry| entry.pending_actions)
        .sum();
    format!(
        "Profil : {}\nDonnées : {}\nActions en attente : {}\n{}",
        status.profile,
        status.data_directory,
        pending,
        accounts_table(&accounts)
    )
}
