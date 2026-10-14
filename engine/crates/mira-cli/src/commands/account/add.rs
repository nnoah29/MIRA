//! `mira account add` : teste puis enregistre un compte, de façon idempotente.

use clap::Args;
use mira_assembly::api::AccountRequest;
use mira_assembly::mira_vocabulary::{IdempotencyKey, MiraError};

use super::input_args::AccountInputArgs;
use super::sync_wait::SyncWaitArgs;
use crate::argument_context::ArgumentContext;

/// Options de `mira account add`.
#[derive(Debug, Args)]
pub struct AddArgs {
    #[command(flatten)]
    pub input: AccountInputArgs,
    /// Clé d'idempotence (générée si absente) ; la rejouer ne crée qu'un compte.
    #[arg(long)]
    pub key: Option<String>,
    #[command(flatten)]
    pub sync: SyncWaitArgs,
}

/// Construit la requête d'ajout de compte.
pub fn request(args: &AddArgs, context: &mut ArgumentContext) -> Result<AccountRequest, MiraError> {
    let (draft, credential) = args.input.draft_and_credential(context)?;
    let key = args.key.clone().unwrap_or_else(|| context.generated_key());
    Ok(AccountRequest::AddAccount {
        draft,
        credential,
        idempotency_key: IdempotencyKey(key),
    })
}
