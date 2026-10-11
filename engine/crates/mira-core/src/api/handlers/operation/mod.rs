//! Domaine « opérations longues ».

use mira_vocabulary::MiraError;

use crate::api::contract::{OperationRequest, Response};
use crate::api::engine::Engine;

/// Route une requête du domaine opérations
pub(crate) async fn route(
    engine: &Engine,
    request: OperationRequest,
) -> Result<Response, MiraError> {
    match request {
        OperationRequest::GetOperation { operation } => handle_get_operation(engine, &operation),
        OperationRequest::CancelOperation { operation } => {
            handle_cancel_operation(engine, &operation)
        }
    }
}

/// Rend la fiche d'une opération
fn handle_get_operation(
    engine: &Engine,
    operation: &mira_vocabulary::OperationId,
) -> Result<Response, MiraError> {
    Ok(Response::Operation(engine.operations.get(operation)?))
}

/// Demande l'annulation d'une opération
fn handle_cancel_operation(
    engine: &Engine,
    operation: &mira_vocabulary::OperationId,
) -> Result<Response, MiraError> {
    engine.operations.cancel(operation)?;
    Ok(Response::Done)
}
