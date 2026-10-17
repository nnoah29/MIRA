//! Secret d'un compte : lu sur l'entrée standard, jamais en argument de ligne de commande.

use clap::{Args, ValueEnum};
use mira_assembly::api::CredentialInput;
use mira_assembly::mira_vocabulary::MiraError;

use crate::argument_context::ArgumentContext;
use crate::argument_error::invalid_argument;

/// Méthode d'authentification choisie par l'utilisateur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum AuthChoice {
    Password,
    AppPassword,
    Oauth,
}

/// Options d'authentification de `account add` et `account test`.
#[derive(Debug, Args)]
pub struct CredentialArgs {
    /// Méthode d'authentification.
    #[arg(long, value_enum, default_value_t = AuthChoice::Password)]
    pub auth: AuthChoice,
    /// Lit le mot de passe sur l'entrée standard (obligatoire hors OAuth).
    #[arg(long)]
    pub password_stdin: bool,
    /// Flux OAuth terminé, pour `--auth oauth`.
    #[arg(long)]
    pub flow: Option<String>,
}

impl CredentialArgs {
    /// Construit l'identifiant du contrat ; le mot de passe n'est lu que si l'option l'autorise.
    pub fn credential(&self, context: &mut ArgumentContext) -> Result<CredentialInput, MiraError> {
        if self.auth == AuthChoice::Oauth {
            let flow = self.flow.clone();
            return flow
                .map(|flow| CredentialInput::OAuthFlow { flow })
                .ok_or_else(|| invalid_argument("--auth oauth demande --flow <flux>"));
        }
        if !self.password_stdin {
            return Err(invalid_argument(
                "le mot de passe se fournit avec --password-stdin",
            ));
        }
        let secret = context.read_secret()?;
        Ok(match self.auth {
            AuthChoice::AppPassword => CredentialInput::AppPassword(secret),
            AuthChoice::Password | AuthChoice::Oauth => CredentialInput::Password(secret),
        })
    }
}
