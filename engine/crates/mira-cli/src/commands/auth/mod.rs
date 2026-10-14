//! Commandes `mira auth …` : OAuth, mot de passe d'application, réauthentification.

pub mod app_password;

use clap::{Args, Subcommand};
use mira_assembly::api::{AuthRequest, Request};
use mira_assembly::mira_vocabulary::{AccountId, MiraError, ProviderId};

use crate::argument_context::ArgumentContext;

/// Début d'un flux OAuth.
#[derive(Debug, Args)]
pub struct OauthStartArgs {
    /// Adresse du compte à connecter.
    pub address: String,
    /// Provider à utiliser (sinon détecté d'après l'adresse).
    #[arg(long)]
    pub provider: Option<String>,
}

/// Flux OAuth en cours.
#[derive(Debug, Args)]
pub struct FlowTarget {
    /// Identifiant du flux rendu par `oauth-start`.
    pub flow: String,
}

/// Compte à réauthentifier.
#[derive(Debug, Args)]
pub struct AccountTarget {
    /// Identifiant du compte.
    pub account: String,
}

/// Commandes du domaine `auth`.
#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    /// Démarre un flux OAuth2 et affiche l'adresse d'autorisation.
    OauthStart(OauthStartArgs),
    /// Attend la redirection et termine un flux OAuth2.
    OauthComplete(FlowTarget),
    /// Annule un flux OAuth2.
    OauthCancel(FlowTarget),
    /// Enregistre un mot de passe d'application lu sur l'entrée standard.
    AppPassword(app_password::AppPasswordArgs),
    /// Relance l'authentification d'un compte.
    Reauthenticate(AccountTarget),
}

/// Traduit une commande `auth` en requête du contrat.
pub fn build_request(
    command: &AuthCommand,
    context: &mut ArgumentContext,
) -> Result<Request, MiraError> {
    let request = match command {
        AuthCommand::OauthStart(args) => AuthRequest::StartOAuth {
            address: args.address.clone(),
            provider: args.provider.as_deref().map(ProviderId::new),
        },
        AuthCommand::OauthComplete(target) => AuthRequest::CompleteOAuth {
            flow: target.flow.clone(),
        },
        AuthCommand::OauthCancel(target) => AuthRequest::CancelOAuth {
            flow: target.flow.clone(),
        },
        AuthCommand::AppPassword(args) => app_password::request(args, context)?,
        AuthCommand::Reauthenticate(target) => AuthRequest::Reauthenticate {
            account: AccountId::from_raw(target.account.as_str()),
        },
    };
    Ok(Request::Auth(request))
}
