//! Options communes à `account test` et `account add` : brouillon de compte et identifiant.

use clap::Args;
use mira_assembly::api::{AccountDraft, CredentialInput};
use mira_assembly::mira_vocabulary::{MiraError, ProviderId};

use super::credential_args::CredentialArgs;
use super::server_args::ServerArgs;
use crate::argument_context::ArgumentContext;

/// Compte tel que l'utilisateur veut le tester ou l'ajouter.
#[derive(Debug, Args)]
pub struct AccountInputArgs {
    /// Adresse e-mail du compte.
    #[arg(long)]
    pub address: String,
    /// Nom affiché (par défaut, l'adresse).
    #[arg(long)]
    pub display_name: Option<String>,
    /// Provider à utiliser (sinon détecté d'après l'adresse).
    #[arg(long)]
    pub provider: Option<String>,
    #[command(flatten)]
    pub servers: ServerArgs,
    #[command(flatten)]
    pub credential: CredentialArgs,
}

impl AccountInputArgs {
    /// Lit le secret puis assemble le brouillon de compte et l'identifiant.
    pub fn draft_and_credential(
        &self,
        context: &mut ArgumentContext,
    ) -> Result<(AccountDraft, CredentialInput), MiraError> {
        let servers = self.servers.settings()?;
        let credential = self.credential.credential(context)?;
        let draft = AccountDraft {
            address: self.address.clone(),
            display_name: self
                .display_name
                .clone()
                .unwrap_or_else(|| self.address.clone()),
            provider: self.provider.as_deref().map(ProviderId::new),
            servers,
            auth: credential.auth_kind(),
        };
        Ok((draft, credential))
    }
}
