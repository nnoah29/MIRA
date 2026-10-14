//! `mira account prepare` : provider retenu et serveurs proposés pour une adresse.

use clap::Args;
use mira_assembly::api::AccountRequest;
use mira_assembly::mira_vocabulary::ProviderId;

/// Options de `mira account prepare`.
#[derive(Debug, Args)]
pub struct PrepareArgs {
    /// Adresse e-mail du futur compte.
    #[arg(long)]
    pub address: String,
    /// Provider à utiliser (sinon détecté d'après l'adresse).
    #[arg(long)]
    pub provider: Option<String>,
}

/// Construit la requête de préparation de compte.
pub fn request(args: &PrepareArgs) -> AccountRequest {
    AccountRequest::PrepareAccount {
        address: args.address.clone(),
        provider: args.provider.as_deref().map(ProviderId::new),
    }
}
