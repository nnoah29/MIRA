//! Commandes `mira mailbox …` : lecture des boîtes, écritures par la file d'actions.

pub mod set_synced;
pub mod write;

use clap::Subcommand;
use mira_assembly::api::{MailboxRequest, Request};
use mira_assembly::mira_vocabulary::MiraError;

use super::targets::AccountArg;
use super::waiting::WaitPolicy;
use crate::argument_context::ArgumentContext;

/// Commandes du domaine `mailbox`.
#[derive(Debug, Subcommand)]
pub enum MailboxCommand {
    /// Liste les boîtes d'un compte avec leurs compteurs.
    List(AccountArg),
    /// Crée une boîte (action mise en file).
    Create(write::CreateArgs),
    /// Renomme une boîte (action mise en file).
    Rename(write::RenameArgs),
    /// Supprime une boîte (action mise en file).
    Delete(write::DeleteArgs),
    /// Vide une boîte (action mise en file).
    Empty(write::EmptyArgs),
    /// Choisit les boîtes synchronisées d'un compte.
    SetSynced(set_synced::SetSyncedArgs),
}

/// Traduit une commande `mailbox` en requête du contrat.
pub fn build_request(
    command: &MailboxCommand,
    context: &ArgumentContext,
) -> Result<Request, MiraError> {
    Ok(match command {
        MailboxCommand::List(target) => Request::Mailbox(MailboxRequest::ListMailboxes {
            account: target.id(),
        }),
        MailboxCommand::Create(args) => write::create_request(args, context),
        MailboxCommand::Rename(args) => write::rename_request(args, context),
        MailboxCommand::Delete(args) => write::delete_request(args, context),
        MailboxCommand::Empty(args) => write::empty_request(args, context),
        MailboxCommand::SetSynced(args) => set_synced::request(args),
    })
}

/// Attente demandée par une commande `mailbox` : seules les écritures peuvent attendre leur rejeu.
pub fn wait_policy(command: &MailboxCommand) -> WaitPolicy {
    match command {
        MailboxCommand::Create(args) => args.wait.policy(),
        MailboxCommand::Rename(args) => args.wait.policy(),
        MailboxCommand::Delete(args) => args.wait.policy(),
        MailboxCommand::Empty(args) => args.wait.policy(),
        MailboxCommand::List(_) | MailboxCommand::SetSynced(_) => WaitPolicy::Immediate,
    }
}
