//! Commandes `mira engine …` : contrat et état du moteur.

use clap::Subcommand;
use mira_assembly::api::{EngineRequest, Request};

/// Commandes du domaine `engine`.
#[derive(Debug, Subcommand)]
pub enum EngineCommand {
    /// Version du contrat d'API et capacités du moteur.
    Info,
    /// Profil, dossier de données, comptes et actions en attente.
    Status,
}

/// Traduit une commande `engine` en requête du contrat.
pub fn build_request(command: &EngineCommand) -> Request {
    match command {
        EngineCommand::Info => Request::Engine(EngineRequest::GetContractInfo),
        EngineCommand::Status => Request::Engine(EngineRequest::GetStatus),
    }
}
