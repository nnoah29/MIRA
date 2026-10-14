//! `mira identity add` : ajoute une identité d'envoi à un compte.

use clap::Args;
use mira_assembly::api::{IdentityRequest, NewIdentity};
use mira_assembly::mira_vocabulary::AccountId;

/// Options de `mira identity add`.
#[derive(Debug, Args)]
pub struct AddArgs {
    /// Identifiant du compte.
    pub account: String,
    /// Adresse d'envoi de l'identité.
    #[arg(long)]
    pub address: String,
    /// Nom affiché.
    #[arg(long)]
    pub display_name: String,
    /// Signature (vide ou absente : aucune signature).
    #[arg(long)]
    pub signature: Option<String>,
    /// Fait de cette identité l'identité par défaut.
    #[arg(long)]
    pub default: bool,
}

/// Construit la requête d'ajout d'identité.
pub fn request(args: &AddArgs) -> IdentityRequest {
    IdentityRequest::AddIdentity(NewIdentity {
        account: AccountId::from_raw(args.account.as_str()),
        address: args.address.clone(),
        display_name: args.display_name.clone(),
        signature: args.signature.clone(),
        is_default: args.default,
    })
}
