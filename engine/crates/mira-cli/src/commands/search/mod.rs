//! Commandes `mira search …` : recherche locale, recherche serveur et lecture de son résultat.

pub mod local;
pub mod server;

use clap::{Args, Subcommand};
use mira_assembly::api::{Request, SearchRequest};
use mira_assembly::mira_vocabulary::{MiraError, OperationId};

use super::waiting::WaitPolicy;

/// Opération de recherche serveur visée.
#[derive(Debug, Args)]
pub struct OperationArg {
    /// Identifiant de l'opération rendu par la recherche serveur.
    pub operation: String,
}

/// Commandes du domaine `search`.
#[derive(Debug, Subcommand)]
pub enum SearchCommand {
    /// Cherche dans le miroir local (instantané, hors ligne).
    Local(local::LocalArgs),
    /// Cherche sur le serveur d'un compte et attend le résultat.
    Server(server::ServerArgs),
    /// Relit le résultat d'une recherche serveur terminée dans le même processus.
    ServerResult(OperationArg),
}

/// Traduit une commande `search` en requête du contrat.
pub fn build_request(command: &SearchCommand) -> Result<Request, MiraError> {
    let request = match command {
        SearchCommand::Local(args) => local::request(args)?,
        SearchCommand::Server(args) => server::request(args),
        SearchCommand::ServerResult(target) => SearchRequest::GetServerSearchResult {
            operation: OperationId::from_raw(target.operation.as_str()),
        },
    };
    Ok(Request::Search(request))
}

/// Attente demandée par une commande `search` : la recherche serveur est toujours attendue.
pub fn wait_policy(command: &SearchCommand) -> WaitPolicy {
    match command {
        SearchCommand::Server(args) => args.wait.policy(),
        SearchCommand::Local(_) | SearchCommand::ServerResult(_) => WaitPolicy::Immediate,
    }
}
