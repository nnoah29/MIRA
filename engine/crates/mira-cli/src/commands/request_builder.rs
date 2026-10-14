//! Table des commandes : chaque commande produit une requête du contrat, routage exhaustif.

use mira_assembly::api::Request;
use mira_assembly::mira_vocabulary::MiraError;

use super::{account, auth, engine, events, identity, operation, provider};
use crate::argument_context::ArgumentContext;
use crate::arguments::Commands;

/// Traduit une commande analysée en requête du contrat (lit stdin si elle porte un secret).
pub fn build_request(
    command: &Commands,
    context: &mut ArgumentContext,
) -> Result<Request, MiraError> {
    match command {
        Commands::Engine(command) => Ok(engine::build_request(command)),
        Commands::Provider(command) => Ok(provider::build_request(command)),
        Commands::Account(command) => account::build_request(command, context),
        Commands::Identity(command) => Ok(identity::build_request(command)),
        Commands::Auth(command) => auth::build_request(command, context),
        Commands::Operation(command) => Ok(operation::build_request(command)),
        Commands::Events(args) => Ok(events::build_request(args)),
    }
}
