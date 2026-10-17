//! Commandes `mira identity …` : identités d'envoi d'un compte.

pub mod add;
pub mod update;

use clap::{Args, Subcommand};
use mira_assembly::api::{IdentityRequest, Request};
use mira_assembly::mira_vocabulary::{AccountId, IdentityId};

/// Compte dont on liste les identités.
#[derive(Debug, Args)]
pub struct AccountTarget {
    /// Identifiant du compte.
    pub account: String,
}

/// Identité visée par une commande.
#[derive(Debug, Args)]
pub struct IdentityTarget {
    /// Identifiant de l'identité.
    pub identity: String,
}

/// Commandes du domaine `identity`.
#[derive(Debug, Subcommand)]
pub enum IdentityCommand {
    /// Liste les identités d'un compte.
    List(AccountTarget),
    /// Ajoute une identité à un compte.
    Add(add::AddArgs),
    /// Modifie le nom ou la signature d'une identité.
    Update(update::UpdateArgs),
    /// Fait d'une identité l'identité par défaut de son compte.
    SetDefault(IdentityTarget),
}

/// Traduit une commande `identity` en requête du contrat.
pub fn build_request(command: &IdentityCommand) -> Request {
    let request = match command {
        IdentityCommand::List(target) => IdentityRequest::ListIdentities {
            account: AccountId::from_raw(target.account.as_str()),
        },
        IdentityCommand::Add(args) => add::request(args),
        IdentityCommand::Update(args) => update::request(args),
        IdentityCommand::SetDefault(target) => IdentityRequest::SetDefaultIdentity {
            identity: IdentityId::from_raw(target.identity.as_str()),
        },
    };
    Request::Identity(request)
}
