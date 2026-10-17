//! Commandes `mira account …` : cycle de vie des comptes.

pub mod add;
pub mod add_oauth;
pub mod credential_args;
pub mod input_args;
pub mod oauth_login;
pub mod prepare;
pub mod server_args;
pub mod sync_wait;
pub mod test_connection;
pub mod update;

use clap::{Args, Subcommand};
use mira_assembly::api::{AccountRequest, Request};
use mira_assembly::mira_vocabulary::{AccountId, MiraError};
use mira_assembly::sync::BackgroundSync;

use crate::argument_context::ArgumentContext;

/// Compte visé par une commande.
#[derive(Debug, Args)]
pub struct AccountTarget {
    /// Identifiant du compte.
    pub account: String,
}

impl AccountTarget {
    /// Rend l'identifiant typé du compte.
    pub fn id(&self) -> AccountId {
        AccountId::from_raw(self.account.as_str())
    }
}

/// Commandes du domaine `account`.
#[derive(Debug, Subcommand)]
pub enum AccountCommand {
    /// Propose provider et serveurs pour une adresse.
    Prepare(prepare::PrepareArgs),
    /// Teste la connexion sans rien enregistrer.
    Test(input_args::AccountInputArgs),
    /// Ajoute un compte (mot de passe lu avec --password-stdin).
    Add(add::AddArgs),
    /// Autorise MIRA par OAuth2 (adresse affichée sur stderr) puis ajoute le compte.
    AddOauth(add_oauth::AddOauthArgs),
    /// Modifie un compte.
    Update(update::UpdateArgs),
    /// Met un compte en pause.
    Pause(AccountTarget),
    /// Reprend un compte en pause.
    Resume(AccountTarget),
    /// Supprime un compte et ses données locales.
    Delete(AccountTarget),
    /// Liste les comptes.
    List,
}

/// Traduit une commande `account` en requête du contrat.
pub fn build_request(
    command: &AccountCommand,
    context: &mut ArgumentContext,
) -> Result<Request, MiraError> {
    let request = match command {
        AccountCommand::Prepare(args) => prepare::request(args),
        AccountCommand::Test(args) => test_connection::request(args, context)?,
        AccountCommand::Add(args) => add::request(args, context)?,
        AccountCommand::AddOauth(args) => add_oauth::request(args, context)?,
        AccountCommand::Update(args) => update::request(args)?,
        AccountCommand::Pause(target) => AccountRequest::PauseAccount {
            account: target.id(),
        },
        AccountCommand::Resume(target) => AccountRequest::ResumeAccount {
            account: target.id(),
        },
        AccountCommand::Delete(target) => AccountRequest::DeleteAccount {
            account: target.id(),
        },
        AccountCommand::List => AccountRequest::ListAccounts,
    };
    Ok(Request::Account(request))
}

/// Synchro de fond permise pendant la commande : seulement si l'ajout attend la première synchro.
pub fn background_mode(command: &AccountCommand, configured: BackgroundSync) -> BackgroundSync {
    match command {
        AccountCommand::Add(args) if args.sync.sync_wait => configured,
        AccountCommand::AddOauth(args) if args.sync.sync_wait => configured,
        AccountCommand::Prepare(_)
        | AccountCommand::Test(_)
        | AccountCommand::Add(_)
        | AccountCommand::AddOauth(_)
        | AccountCommand::Update(_)
        | AccountCommand::Pause(_)
        | AccountCommand::Resume(_)
        | AccountCommand::Delete(_)
        | AccountCommand::List => BackgroundSync::Off,
    }
}
