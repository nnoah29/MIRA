//! Commandes `mira provider …` : catalogue et détection (lecture du registre, sans ouvrir de profil).

pub mod detect;
pub mod labels;
pub mod list;

use clap::Subcommand;
use mira_assembly::ProviderRegistry;
use mira_assembly::api::{ProviderRequest, Request};

use crate::cli_io::CliIo;
use crate::output::OutputMode;

/// Commandes du domaine `provider`.
#[derive(Debug, Subcommand)]
pub enum ProviderCommand {
    /// Liste les providers connus, le repli en dernier.
    List,
    /// Détecte le provider d'une adresse et affiche ses serveurs.
    Detect {
        /// Adresse e-mail à analyser.
        address: String,
    },
}

/// Aiguille une commande du domaine `provider` vers son gestionnaire.
pub fn run_provider(
    command: ProviderCommand,
    registry: &ProviderRegistry,
    mode: OutputMode,
    io: &mut CliIo,
) -> i32 {
    match command {
        ProviderCommand::List => list::run_list(registry, mode, io),
        ProviderCommand::Detect { address } => detect::run_detect(registry, &address, mode, io),
    }
}

/// Traduit une commande `provider` en requête du contrat.
pub fn build_request(command: &ProviderCommand) -> Request {
    match command {
        ProviderCommand::List => Request::Provider(ProviderRequest::ListProviders),
        ProviderCommand::Detect { address } => Request::Provider(ProviderRequest::DetectProvider {
            address: address.clone(),
        }),
    }
}
