//! `mira auth app-password` : le mot de passe est lu sur l'entrée standard, jamais en argument.

use clap::Args;
use mira_assembly::api::AuthRequest;
use mira_assembly::mira_vocabulary::{AccountId, MiraError};

use crate::argument_context::ArgumentContext;
use crate::argument_error::invalid_argument;

/// Options de `mira auth app-password`.
#[derive(Debug, Args)]
pub struct AppPasswordArgs {
    /// Identifiant du compte.
    pub account: String,
    /// Lit le mot de passe d'application sur l'entrée standard (obligatoire).
    #[arg(long)]
    pub password_stdin: bool,
}

/// Construit la requête de rangement du mot de passe d'application.
pub fn request(
    args: &AppPasswordArgs,
    context: &mut ArgumentContext,
) -> Result<AuthRequest, MiraError> {
    if !args.password_stdin {
        return Err(invalid_argument(
            "le mot de passe se fournit avec --password-stdin",
        ));
    }
    Ok(AuthRequest::StoreAppPassword {
        account: AccountId::from_raw(args.account.as_str()),
        password: context.read_secret()?,
    })
}
