//! `mira account add-oauth` : autorisation OAuth2 et ajout du compte dans un seul processus.

use clap::Args;
use mira_assembly::api::{AccountDraft, AccountRequest, CredentialInput};
use mira_assembly::mira_vocabulary::{AuthMethodKind, IdempotencyKey, MiraError, ProviderId};

use super::server_args::ServerArgs;
use super::sync_wait::SyncWaitArgs;
use crate::argument_context::ArgumentContext;

/// Options de `mira account add-oauth`.
#[derive(Debug, Args)]
pub struct AddOauthArgs {
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
    /// Clé d'idempotence (générée si absente).
    #[arg(long)]
    pub key: Option<String>,
    #[command(flatten)]
    pub sync: SyncWaitArgs,
}

/// Requête d'ajout dont le flux est à remplir une fois l'autorisation reçue.
pub fn request(
    args: &AddOauthArgs,
    context: &ArgumentContext,
) -> Result<AccountRequest, MiraError> {
    let draft = AccountDraft {
        address: args.address.clone(),
        display_name: args
            .display_name
            .clone()
            .unwrap_or_else(|| args.address.clone()),
        provider: args.provider.as_deref().map(ProviderId::new),
        servers: args.servers.settings()?,
        auth: AuthMethodKind::OAuth2,
    };
    let key = args.key.clone().unwrap_or_else(|| context.generated_key());
    Ok(AccountRequest::AddAccount {
        draft,
        credential: CredentialInput::OAuthFlow {
            flow: String::new(),
        },
        idempotency_key: IdempotencyKey(key),
    })
}
