//! `mira search server` : recherche sur le serveur d'un compte (opération réseau explicite).

use clap::Args;
use mira_assembly::api::SearchRequest;
use mira_assembly::mira_vocabulary::AccountId;

use crate::commands::waiting::OperationWaitArgs;

/// Options de `mira search server`.
#[derive(Debug, Args)]
pub struct ServerArgs {
    /// Identifiant du compte à interroger.
    pub account: String,
    /// Mots cherchés par le serveur.
    #[arg(required = true)]
    pub words: Vec<String>,
    #[command(flatten)]
    pub wait: OperationWaitArgs,
}

/// Construit la requête de recherche serveur.
pub fn request(args: &ServerArgs) -> SearchRequest {
    SearchRequest::SearchServer {
        account: AccountId::from_raw(args.account.as_str()),
        query: args.words.join(" "),
    }
}
