//! Commandes `mira queue …` : file d'actions locale d'un compte.

use clap::{Args, Subcommand, ValueEnum};
use mira_assembly::api::{QueueRequest, Request};
use mira_assembly::mira_vocabulary::ActionState;

use super::targets::{AccountArg, ActionArg};
use super::waiting::{WaitArgs, WaitPolicy};

/// États d'une action de la file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum StateName {
    /// En attente de son tour.
    Pending,
    /// En cours de rejeu vers le serveur.
    InProgress,
    /// Appliquée par le serveur.
    Succeeded,
    /// Refusée ou abandonnée.
    Failed,
    /// Annulée avant son rejeu.
    Cancelled,
    /// Résultat inconnu (envoi interrompu).
    Uncertain,
}

/// Options de `mira queue list`.
#[derive(Debug, Args)]
pub struct ListArgs {
    #[command(flatten)]
    pub account: AccountArg,
    /// Ne garde que ces états (répétable) ; sans option, toutes les actions.
    #[arg(long = "state", value_enum)]
    pub states: Vec<StateName>,
}

/// Options de `mira queue retry`.
#[derive(Debug, Args)]
pub struct RetryArgs {
    #[command(flatten)]
    pub action: ActionArg,
    #[command(flatten)]
    pub wait: WaitArgs,
}

/// Commandes du domaine `queue`.
#[derive(Debug, Subcommand)]
pub enum QueueCommand {
    /// Liste les actions d'un compte dans l'ordre de la file.
    List(ListArgs),
    /// Relance une action échouée.
    Retry(RetryArgs),
    /// Annule une action encore en attente.
    Cancel(ActionArg),
}

/// État du contrat correspondant au nom saisi.
fn action_state(name: StateName) -> ActionState {
    match name {
        StateName::Pending => ActionState::Pending,
        StateName::InProgress => ActionState::InProgress,
        StateName::Succeeded => ActionState::Succeeded,
        StateName::Failed => ActionState::Failed,
        StateName::Cancelled => ActionState::Cancelled,
        StateName::Uncertain => ActionState::Uncertain,
    }
}

/// Traduit une commande `queue` en requête du contrat.
pub fn build_request(command: &QueueCommand) -> Request {
    let request = match command {
        QueueCommand::List(args) => QueueRequest::ListActions {
            account: args.account.id(),
            states: args.states.iter().copied().map(action_state).collect(),
        },
        QueueCommand::Retry(args) => QueueRequest::RetryAction {
            action: args.action.id(),
        },
        QueueCommand::Cancel(target) => QueueRequest::CancelAction {
            action: target.id(),
        },
    };
    Request::Queue(request)
}

/// Attente demandée par une commande `queue` : seule la relance peut attendre son rejeu.
pub fn wait_policy(command: &QueueCommand) -> WaitPolicy {
    match command {
        QueueCommand::Retry(args) => args.wait.policy(),
        QueueCommand::List(_) | QueueCommand::Cancel(_) => WaitPolicy::Immediate,
    }
}
