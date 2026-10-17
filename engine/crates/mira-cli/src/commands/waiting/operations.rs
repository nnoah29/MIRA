//! Attente d'une opération longue (téléchargement, enregistrement, recherche serveur).

use std::time::Duration;

use mira_assembly::api::{Engine, OperationRequest, Request, Response, SearchRequest};
use mira_assembly::mira_vocabulary::{MiraError, OperationId};
use mira_assembly::operations::{OperationInfo, OperationStatus};
use tokio::time::{Instant, sleep};

use super::failure::{error_from_code, timeout_error};

const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Interroge l'opération jusqu'à ce qu'elle ne soit plus en cours ou que le délai s'écoule.
async fn poll_operation(
    engine: &Engine,
    operation: &OperationId,
    seconds: u64,
) -> Result<OperationInfo, MiraError> {
    let deadline = Instant::now() + Duration::from_secs(seconds);
    loop {
        let request = OperationRequest::GetOperation {
            operation: operation.clone(),
        };
        let answer = engine.execute(Request::Operation(request)).await?;
        if let Response::Operation(info) = answer
            && !matches!(info.status, OperationStatus::Running { .. })
        {
            return Ok(info);
        }
        if Instant::now() >= deadline {
            return Err(timeout_error(seconds));
        }
        sleep(POLL_INTERVAL).await;
    }
}

/// Résultat d'une recherche serveur terminée.
async fn server_search_result(
    engine: &Engine,
    operation: OperationId,
) -> Result<Response, MiraError> {
    let request = SearchRequest::GetServerSearchResult { operation };
    engine.execute(Request::Search(request)).await
}

/// Attend la fin de l'opération : la recherche serveur rend ses résultats, les autres leur fiche finale.
pub async fn finish_operation(
    engine: &Engine,
    started: OperationInfo,
    seconds: u64,
) -> Result<Response, MiraError> {
    let done = poll_operation(engine, &started.id, seconds).await?;
    match &done.status {
        OperationStatus::Failed { code } => Err(error_from_code(code)),
        OperationStatus::Finished if done.kind == "search_server" => {
            server_search_result(engine, done.id).await
        }
        OperationStatus::Finished
        | OperationStatus::Cancelled
        | OperationStatus::Running { .. } => Ok(Response::Operation(done)),
    }
}
