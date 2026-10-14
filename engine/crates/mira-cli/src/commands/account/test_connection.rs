//! `mira account test` : essaie la connexion sans rien enregistrer.

use mira_assembly::api::AccountRequest;
use mira_assembly::mira_vocabulary::MiraError;

use super::input_args::AccountInputArgs;
use crate::argument_context::ArgumentContext;

/// Construit la requête de test de connexion.
pub fn request(
    args: &AccountInputArgs,
    context: &mut ArgumentContext,
) -> Result<AccountRequest, MiraError> {
    let (draft, credential) = args.draft_and_credential(context)?;
    Ok(AccountRequest::TestConnection { draft, credential })
}
