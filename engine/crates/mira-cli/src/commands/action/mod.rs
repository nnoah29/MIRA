//! Commandes `mira action …` : actions sur les messages (lot) et annulation.

pub mod apply;
pub mod kind;
pub mod selection;

use clap::Subcommand;
use mira_assembly::api::{ActionRequest, Request};
use mira_assembly::mira_vocabulary::MiraError;

use super::targets::ActionArg;
use super::waiting::WaitPolicy;
use crate::argument_context::ArgumentContext;

/// Commandes du domaine `action`.
#[derive(Debug, Subcommand)]
pub enum ActionCommand {
    /// Applique une action à des messages choisis par liste ou par requête.
    Apply(apply::ApplyArgs),
    /// Annule une action encore en attente (dans son délai d'annulation).
    Undo(ActionArg),
}

/// Traduit une commande `action` en requête du contrat.
pub fn build_request(
    command: &ActionCommand,
    context: &ArgumentContext,
) -> Result<Request, MiraError> {
    let request = match command {
        ActionCommand::Apply(args) => apply::request(args, context)?,
        ActionCommand::Undo(target) => ActionRequest::UndoAction {
            action: target.id(),
        },
    };
    Ok(Request::Action(request))
}

/// Attente demandée par une commande `action` : seule l'application peut attendre son rejeu.
pub fn wait_policy(command: &ActionCommand) -> WaitPolicy {
    match command {
        ActionCommand::Apply(args) => args.wait.policy(),
        ActionCommand::Undo(_) => WaitPolicy::Immediate,
    }
}
