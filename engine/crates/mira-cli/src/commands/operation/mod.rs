//! Commandes `mira operation …` : suivi et annulation des opérations longues.

use clap::{Args, Subcommand};
use mira_assembly::api::{OperationRequest, Request};
use mira_assembly::mira_vocabulary::OperationId;

/// Identifiant d'une opération longue.
#[derive(Debug, Args)]
pub struct OperationTarget {
    /// Identifiant de l'opération.
    pub operation: String,
}

/// Commandes du domaine `operation`.
#[derive(Debug, Subcommand)]
pub enum OperationCommand {
    /// Affiche la nature et l'avancement d'une opération.
    Get(OperationTarget),
    /// Demande l'annulation d'une opération.
    Cancel(OperationTarget),
}

/// Traduit une commande `operation` en requête du contrat.
pub fn build_request(command: &OperationCommand) -> Request {
    match command {
        OperationCommand::Get(target) => Request::Operation(OperationRequest::GetOperation {
            operation: OperationId::from_raw(target.operation.as_str()),
        }),
        OperationCommand::Cancel(target) => Request::Operation(OperationRequest::CancelOperation {
            operation: OperationId::from_raw(target.operation.as_str()),
        }),
    }
}
