//! `mira identity update` : modifie le nom affiché ou la signature d'une identité.

use clap::Args;
use mira_assembly::api::IdentityRequest;
use mira_assembly::mira_vocabulary::IdentityId;

/// Options de `mira identity update`.
#[derive(Debug, Args)]
pub struct UpdateArgs {
    /// Identifiant de l'identité.
    pub identity: String,
    /// Nouveau nom affiché.
    #[arg(long)]
    pub display_name: Option<String>,
    /// Nouvelle signature.
    #[arg(long)]
    pub signature: Option<String>,
}

/// Construit la requête de modification d'identité.
pub fn request(args: &UpdateArgs) -> IdentityRequest {
    IdentityRequest::UpdateIdentity {
        identity: IdentityId::from_raw(args.identity.as_str()),
        display_name: args.display_name.clone(),
        signature: args.signature.clone(),
    }
}
